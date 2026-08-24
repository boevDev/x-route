#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Read};
use std::net::IpAddr;
use std::os::windows::process::CommandExt;
use std::process::{Command, Stdio};
use tauri::{Emitter, Manager};

const CREATE_NO_WINDOW: u32 = 0x08000000;

const PROGRESS_EVENT: &str = "dns-progress";
const STEP_PREFIX: &str = "__XR_STEP__:";

const RESOLVE_ROUTE_HEAD: &str = r#"
$route = Get-NetRoute -DestinationPrefix '0.0.0.0/0' -ErrorAction SilentlyContinue |
    Sort-Object -Property RouteMetric |
    Select-Object -First 1;
if (-not $route) {
    throw 'Не найден активный сетевой маршрут (нет подключения к сети)';
}
"#;

const STATUS_TAIL: &str = r#"
$adapter = Get-NetAdapter -InterfaceIndex $route.InterfaceIndex;
$dns = @(Get-DnsClientServerAddress -InterfaceIndex $route.InterfaceIndex |
    Select-Object AddressFamily, ServerAddresses);
[PSCustomObject]@{
    InterfaceIndex = $adapter.ifIndex
    InterfaceName  = $adapter.Name
    Dns            = $dns
} | ConvertTo-Json -Compress -Depth 4
"#;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProgressEvent<'a> {
    stage: &'a str,
    operation: &'a str,
    detail: Option<&'a str>,
}

fn emit_progress(app: &tauri::AppHandle, stage: &str, operation: &str, detail: Option<&str>) {
    let _ = app.emit(
        PROGRESS_EVENT,
        ProgressEvent {
            stage,
            operation,
            detail,
        },
    );
}

enum ToggleOp {
    Enable,
    Disable,
}

impl ToggleOp {
    fn as_str(&self) -> &'static str {
        match self {
            ToggleOp::Enable => "enable",
            ToggleOp::Disable => "disable",
        }
    }

    fn step_marker(&self) -> &'static str {
        match self {
            ToggleOp::Enable => "apply",
            ToggleOp::Disable => "reset",
        }
    }

    fn stage_for(&self, marker: &str) -> Option<&'static str> {
        match marker {
            "resolve" => Some("resolving"),
            "apply" if matches!(self, ToggleOp::Enable) => Some("applying"),
            "reset" if matches!(self, ToggleOp::Disable) => Some("resetting"),
            "clear" => Some("clearing"),
            "verify" => Some("verifying"),
            _ => None,
        }
    }
}

fn fail_toggle(app: &tauri::AppHandle, op: &ToggleOp, msg: String) -> String {
    emit_progress(app, "error", op.as_str(), Some(&msg));
    msg
}

fn run_step_marker_line(marker: &str) -> String {
    format!("[Console]::Out.WriteLine('{STEP_PREFIX}{marker}'); [Console]::Out.Flush();")
}

fn run_powershell_streamed<F>(
    app: &tauri::AppHandle,
    operation: &str,
    on_marker: F,
    script: &str,
) -> Result<String, String>
where
    F: Fn(&str) -> Option<&'static str>,
{
    let wrapped = format!(
        "[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; $ErrorActionPreference = 'Stop'; {script}"
    );

    let mut child = Command::new("powershell")
        .creation_flags(CREATE_NO_WINDOW)
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &wrapped,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Не удалось запустить PowerShell: {e}"))?;

    let stderr_reader = child.stderr.take().map(|mut stderr| {
        std::thread::spawn(move || {
            let mut buf = String::new();
            let _ = stderr.read_to_string(&mut buf);
            buf
        })
    });

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "Не удалось получить вывод PowerShell".to_string())?;

    let mut payload_lines: Vec<String> = vec![];
    for line in BufReader::new(stdout).lines() {
        let line = line.map_err(|e| format!("Ошибка чтения вывода PowerShell: {e}"))?;
        match line.strip_prefix(STEP_PREFIX) {
            Some(marker) => {
                if let Some(stage) = on_marker(marker.trim()) {
                    emit_progress(app, stage, operation, None);
                }
            }
            None => payload_lines.push(line),
        }
    }

    let status = child
        .wait()
        .map_err(|e| format!("Не удалось дождаться PowerShell: {e}"))?;
    let stderr = stderr_reader
        .and_then(|handle| handle.join().ok())
        .unwrap_or_default();

    let stderr_trimmed = stderr.trim().to_string();

    if !status.success() || !stderr_trimmed.is_empty() {
        return Err(if stderr_trimmed.is_empty() {
            "PowerShell завершился с ошибкой (проверь права администратора)".to_string()
        } else if stderr_trimmed.contains("Access is denied") || stderr_trimmed.contains("отказано")
        {
            "Отказано в доступе. Приложению нужны права администратора для смены DNS.".to_string()
        } else {
            stderr_trimmed
        });
    }

    Ok(payload_lines.join("\n").trim().to_string())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct DnsStatus {
    #[serde(rename = "interfaceName")]
    interface_name: String,
    #[serde(rename = "interfaceIndex")]
    interface_index: u32,
    ipv4: Vec<String>,
    ipv6: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RawDnsEntry {
    #[serde(rename = "AddressFamily")]
    address_family: u32,
    #[serde(rename = "ServerAddresses")]
    server_addresses: RawAddresses,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RawAddresses {
    Many(Vec<String>),
    One(String),
    None,
}

impl RawAddresses {
    fn into_vec(self) -> Vec<String> {
        match self {
            RawAddresses::Many(v) => v,
            RawAddresses::One(s) if !s.is_empty() => vec![s],
            _ => vec![],
        }
    }
}

#[derive(Debug, Deserialize)]
struct StatusPayload {
    #[serde(rename = "InterfaceIndex")]
    interface_index: u32,
    #[serde(rename = "InterfaceName")]
    interface_name: String,
    #[serde(rename = "Dns", default)]
    dns: Vec<RawDnsEntry>,
}

fn run_powershell(script: &str) -> Result<String, String> {
    let wrapped = format!(
        "[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; $ErrorActionPreference = 'Stop'; {script}"
    );

    let output = Command::new("powershell")
        .creation_flags(CREATE_NO_WINDOW)
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &wrapped,
        ])
        .output()
        .map_err(|e| format!("Не удалось запустить PowerShell: {e}"))?;

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if !output.status.success() || !stderr.is_empty() {
        return Err(if stderr.is_empty() {
            "PowerShell завершился с ошибкой (проверь права администратора)".to_string()
        } else if stderr.contains("Access is denied") || stderr.contains("отказано") {
            "Отказано в доступе. Приложению нужны права администратора для смены DNS.".to_string()
        } else {
            stderr
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn parse_status(raw: &str) -> Result<DnsStatus, String> {
    let payload: StatusPayload =
        serde_json::from_str(raw).map_err(|e| format!("Не удалось разобрать ответ ОС: {e}"))?;

    let mut ipv4 = vec![];
    let mut ipv6 = vec![];
    for entry in payload.dns {
        let addrs = entry.server_addresses.into_vec();
        match entry.address_family {
            2 => ipv4 = addrs,
            23 => ipv6 = addrs,
            _ => {}
        }
    }

    Ok(DnsStatus {
        interface_name: payload.interface_name,
        interface_index: payload.interface_index,
        ipv4,
        ipv6,
    })
}

fn get_dns_status_sync() -> Result<DnsStatus, String> {
    let script = format!("{RESOLVE_ROUTE_HEAD}{STATUS_TAIL}");
    let raw = run_powershell(&script)?;
    if raw.is_empty() {
        return Err("PowerShell вернул пустой ответ".to_string());
    }
    parse_status(&raw)
}

fn set_dns_sync(
    app: &tauri::AppHandle,
    enable: bool,
    ipv4: Vec<String>,
    ipv6: Vec<String>,
) -> Result<DnsStatus, String> {
    let op = if enable {
        ToggleOp::Enable
    } else {
        ToggleOp::Disable
    };

    let set_action = if enable {
        let mut validated: Vec<String> = Vec::with_capacity(ipv4.len() + ipv6.len());
        for ip in ipv4.iter().chain(ipv6.iter()) {
            match ip.parse::<IpAddr>() {
                Ok(parsed) => validated.push(format!("'{parsed}'")),
                Err(_) => {
                    let msg = format!("Некорректный IP-адрес: {ip}");
                    return Err(fail_toggle(app, &op, msg));
                }
            }
        }

        if validated.is_empty() {
            let msg = "Список целевых DNS-адресов пуст".to_string();
            return Err(fail_toggle(app, &op, msg));
        }

        format!(
            "Set-DnsClientServerAddress -InterfaceIndex $route.InterfaceIndex -ServerAddresses @({})",
            validated.join(",")
        )
    } else {
        "Set-DnsClientServerAddress -InterfaceIndex $route.InterfaceIndex -ResetServerAddresses"
            .to_string()
    };

    let script = format!(
        "{}\n{}\n{}\n{};\ntry {{ {}\nClear-DnsClientCache }} catch {{}};\n{}\n{}",
        run_step_marker_line("resolve"),
        RESOLVE_ROUTE_HEAD,
        run_step_marker_line(op.step_marker()),
        set_action,
        run_step_marker_line("clear"),
        run_step_marker_line("verify"),
        STATUS_TAIL
    );

    let raw = run_powershell_streamed(app, op.as_str(), |marker| op.stage_for(marker), &script)?;

    if raw.is_empty() {
        let msg = "PowerShell вернул пустой ответ".to_string();
        return Err(fail_toggle(app, &op, msg));
    }

    match parse_status(&raw) {
        Ok(status) => {
            emit_progress(app, "done", op.as_str(), None);
            Ok(status)
        }
        Err(e) => Err(fail_toggle(app, &op, e)),
    }
}

async fn run_blocking<T, F>(task: F) -> Result<T, String>
where
    F: FnOnce() -> Result<T, String> + Send + 'static,
    T: Send + 'static,
{
    match tauri::async_runtime::spawn_blocking(task).await {
        Ok(inner) => inner,
        Err(e) => Err(format!("Фоновая задача прервана: {e}")),
    }
}

#[tauri::command]
async fn get_dns_status() -> Result<DnsStatus, String> {
    run_blocking(get_dns_status_sync).await
}

#[tauri::command]
async fn set_dns(
    app: tauri::AppHandle,
    enable: bool,
    ipv4: Vec<String>,
    ipv6: Vec<String>,
) -> Result<DnsStatus, String> {
    run_blocking(move || set_dns_sync(&app, enable, ipv4, ipv6)).await
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn main() {
    use tauri::menu::{Menu, MenuItem};
    use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
    use tauri::WindowEvent;
    use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            show_main_window(app);
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .invoke_handler(tauri::generate_handler![get_dns_status, set_dns])
        .setup(|app| {
            // Реестр (проверка/включение автозапуска) — вне критического пути старта
            let handle = app.handle().clone();
            tauri::async_runtime::spawn_blocking(move || {
                let autostart = handle.autolaunch();
                if !autostart.is_enabled().unwrap_or(false) {
                    let _ = autostart.enable();
                }
            });

            // Трей-меню: показать окно / выйти полностью
            let show_item = MenuItem::with_id(app, "show", "Показать X-Route", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Выход", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("X-Route")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => show_main_window(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        show_main_window(&app);
                    }
                })
                .build(app)?;

            // Если запущены автозапуском (--minimized) — остаёмся в трее.
            // Иначе показываем окно сразу (в конфиге оно visible: false по умолчанию)
            let launched_minimized = std::env::args().any(|a| a == "--minimized");
            if !launched_minimized {
                show_main_window(&app.app_handle());
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            // Клик по крестику сворачивает в трей, а не закрывает приложение
            if let WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
