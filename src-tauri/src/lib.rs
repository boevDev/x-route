mod commands;
mod dns;
mod powershell;
mod progress;
mod tray;

use tauri::Manager;
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

fn enable_autostart(handle: tauri::AppHandle) {
    let autostart = handle.autolaunch();
    if !autostart.is_enabled().unwrap_or(false) {
        let _ = autostart.enable();
    }
}

fn launched_minimized() -> bool {
    std::env::args().any(|arg| arg == "--minimized")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            tray::show_main_window(app);
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .invoke_handler(tauri::generate_handler![
            commands::get_dns_status,
            commands::set_dns
        ])
        .setup(|app| {
            // Реестр (проверка/включение автозапуска) — вне критического пути старта
            let handle = app.handle().clone();
            tauri::async_runtime::spawn_blocking(move || enable_autostart(handle));

            // Трей-меню: показать окно / выйти полностью
            tray::install(app)?;

            // Если запущены автозапуском (--minimized) — остаёмся в трее.
            // Иначе показываем окно сразу (в конфиге оно visible: false по умолчанию)
            if !launched_minimized() {
                tray::show_main_window(&app.app_handle());
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            // Клик по крестику сворачивает в трей, а не закрывает приложение
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
