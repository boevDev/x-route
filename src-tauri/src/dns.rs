use std::net::IpAddr;

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::powershell::{run_powershell, run_powershell_streamed, step_marker_line};
use crate::progress::emit_progress;

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DnsStatus {
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

fn fail_toggle(app: &AppHandle, op: &ToggleOp, msg: String) -> String {
    emit_progress(app, "error", op.as_str(), Some(&msg));
    msg
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

pub fn get_status() -> Result<DnsStatus, String> {
    let script = format!("{RESOLVE_ROUTE_HEAD}{STATUS_TAIL}");
    let raw = run_powershell(&script)?;
    if raw.is_empty() {
        return Err("PowerShell вернул пустой ответ".to_string());
    }
    parse_status(&raw)
}

pub fn set_dns(
    app: &AppHandle,
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
        step_marker_line("resolve"),
        RESOLVE_ROUTE_HEAD,
        step_marker_line(op.step_marker()),
        set_action,
        step_marker_line("clear"),
        step_marker_line("verify"),
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
