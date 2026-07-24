#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::os::windows::process::CommandExt;
use std::process::Command;

const CREATE_NO_WINDOW: u32 = 0x08000000;

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

fn resolve_active_interface() -> Result<(u32, String), String> {
    let script = r#"
        $route = Get-NetRoute -DestinationPrefix '0.0.0.0/0' -ErrorAction SilentlyContinue |
            Sort-Object -Property RouteMetric |
            Select-Object -First 1;
        if (-not $route) {
            throw 'Не найден активный сетевой маршрут (нет подключения к сети)';
        }
        $adapter = Get-NetAdapter -InterfaceIndex $route.InterfaceIndex;
        [PSCustomObject]@{
            InterfaceIndex = $adapter.ifIndex
            InterfaceName  = $adapter.Name
        } | ConvertTo-Json -Compress
    "#;

    let raw = run_powershell(script)?;

    #[derive(Deserialize)]
    struct AdapterInfo {
        #[serde(rename = "InterfaceIndex")]
        interface_index: u32,
        #[serde(rename = "InterfaceName")]
        interface_name: String,
    }

    let info: AdapterInfo =
        serde_json::from_str(&raw).map_err(|e| format!("Не удалось разобрать ответ ОС: {e}"))?;

    Ok((info.interface_index, info.interface_name))
}

fn read_dns_for_interface(interface_index: u32) -> Result<(Vec<String>, Vec<String>), String> {
    let script = format!(
        r#"
        Get-DnsClientServerAddress -InterfaceIndex {} |
            Select-Object AddressFamily, ServerAddresses |
            ConvertTo-Json -Compress
        "#,
        interface_index
    );

    let raw = run_powershell(&script)?;
    if raw.is_empty() {
        return Ok((vec![], vec![]));
    }

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum RawEntries {
        Many(Vec<RawDnsEntry>),
        One(RawDnsEntry),
    }

    let entries: RawEntries =
        serde_json::from_str(&raw).map_err(|e| format!("Не удалось разобрать DNS-записи: {e}"))?;
    let entries = match entries {
        RawEntries::Many(v) => v,
        RawEntries::One(e) => vec![e],
    };

    let mut ipv4 = vec![];
    let mut ipv6 = vec![];
    for entry in entries {
        let addrs = entry.server_addresses.into_vec();
        match entry.address_family {
            2 => ipv4 = addrs,
            23 => ipv6 = addrs,
            _ => {}
        }
    }

    Ok((ipv4, ipv6))
}

#[tauri::command]
fn get_dns_status() -> Result<DnsStatus, String> {
    let (interface_index, interface_name) = resolve_active_interface()?;
    let (ipv4, ipv6) = read_dns_for_interface(interface_index)?;

    Ok(DnsStatus {
        interface_name,
        interface_index,
        ipv4,
        ipv6,
    })
}

#[tauri::command]
fn set_dns(enable: bool, ipv4: Vec<String>, ipv6: Vec<String>) -> Result<DnsStatus, String> {
    let (interface_index, _) = resolve_active_interface()?;

    let script = if enable {
        let mut validated: Vec<String> = vec![];
        for ip in ipv4.iter().chain(ipv6.iter()) {
            match ip.parse::<IpAddr>() {
                Ok(parsed) => validated.push(parsed.to_string()),
                Err(_) => return Err(format!("Некорректный IP-адрес: {ip}")),
            }
        }

        if validated.is_empty() {
            return Err("Список целевых DNS-адресов пуст".to_string());
        }

        let joined = validated
            .iter()
            .map(|ip| format!("'{ip}'"))
            .collect::<Vec<_>>()
            .join(",");

        format!(
            "Set-DnsClientServerAddress -InterfaceIndex {interface_index} -ServerAddresses @({joined})"
        )
    } else {
        format!("Set-DnsClientServerAddress -InterfaceIndex {interface_index} -ResetServerAddresses")
    };

    run_powershell(&script)?;
    let _ = run_powershell("Clear-DnsClientCache");

    get_dns_status()
}

fn main() {
    use tauri::menu::{Menu, MenuItem};
    use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
    use tauri::{Manager, WindowEvent};
    use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .invoke_handler(tauri::generate_handler![get_dns_status, set_dns])
        .setup(|app| {
            // Включаем автозапуск при первом старте, если ещё не включён
            let autostart = app.autolaunch();
            if !autostart.is_enabled().unwrap_or(false) {
                let _ = autostart.enable();
            }

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
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
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
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                })
                .build(app)?;

            // Если запущены автозапуском (--minimized) — остаёмся в трее.
            // Иначе показываем окно сразу (в конфиге оно visible: false по умолчанию)
            let launched_minimized = std::env::args().any(|a| a == "--minimized");
            if !launched_minimized {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
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