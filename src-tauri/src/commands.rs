use tauri::AppHandle;

use crate::dns::{self, DnsStatus};

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
pub async fn get_dns_status() -> Result<DnsStatus, String> {
    run_blocking(dns::get_status).await
}

#[tauri::command]
pub async fn set_dns(
    app: AppHandle,
    enable: bool,
    ipv4: Vec<String>,
    ipv6: Vec<String>,
) -> Result<DnsStatus, String> {
    run_blocking(move || dns::set_dns(&app, enable, ipv4, ipv6)).await
}
