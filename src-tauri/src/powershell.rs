use std::io::{BufRead, BufReader, Read};
use std::os::windows::process::CommandExt;
use std::process::{Command, Stdio};

use tauri::AppHandle;

use crate::progress::emit_progress;

const CREATE_NO_WINDOW: u32 = 0x08000000;
const STEP_PREFIX: &str = "__XR_STEP__:";

fn wrap_script(script: &str) -> String {
    format!(
        "[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; $ErrorActionPreference = 'Stop'; {script}"
    )
}

fn powershell_command(script: &str) -> Command {
    let mut command = Command::new("powershell");
    command
        .creation_flags(CREATE_NO_WINDOW)
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ]);
    command
}

fn map_failure(stderr: String) -> String {
    if stderr.is_empty() {
        "PowerShell завершился с ошибкой (проверь права администратора)".to_string()
    } else if stderr.contains("Access is denied") || stderr.contains("отказано") {
        "Отказано в доступе. Приложению нужны права администратора для смены DNS.".to_string()
    } else {
        stderr
    }
}

pub fn step_marker_line(marker: &str) -> String {
    format!("[Console]::Out.WriteLine('{STEP_PREFIX}{marker}'); [Console]::Out.Flush();")
}

pub fn run_powershell(script: &str) -> Result<String, String> {
    let output = powershell_command(&wrap_script(script))
        .output()
        .map_err(|e| format!("Не удалось запустить PowerShell: {e}"))?;

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if !output.status.success() || !stderr.is_empty() {
        return Err(map_failure(stderr));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn run_powershell_streamed<F>(
    app: &AppHandle,
    operation: &str,
    on_marker: F,
    script: &str,
) -> Result<String, String>
where
    F: Fn(&str) -> Option<&'static str>,
{
    let mut child = powershell_command(&wrap_script(script))
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
        return Err(map_failure(stderr_trimmed));
    }

    Ok(payload_lines.join("\n").trim().to_string())
}
