use serde::Serialize;
use std::collections::HashSet;
use std::os::windows::process::CommandExt;
use std::process::Command;
use winreg::enums::{
    HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY,
};
use winreg::RegKey;

const CREATE_NO_WINDOW: u32 = 0x08000000;
const UNINSTALL_PATH: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall";

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct InstalledApp {
    name: String,
    version: Option<String>,
    publisher: Option<String>,
    install_date: Option<String>,
    has_uninstall_command: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InstallResult {
    package_id: String,
    success: bool,
    exit_code: Option<i32>,
    message: String,
}

fn read_optional_string(key: &RegKey, name: &str) -> Option<String> {
    key.get_value::<String, _>(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn collect_from_hive(hive: RegKey, flags: u32, apps: &mut Vec<InstalledApp>) {
    let Ok(uninstall_key) = hive.open_subkey_with_flags(UNINSTALL_PATH, KEY_READ | flags) else {
        return;
    };

    for subkey_name in uninstall_key.enum_keys().filter_map(Result::ok) {
        let Ok(app_key) = uninstall_key.open_subkey_with_flags(&subkey_name, KEY_READ | flags) else {
            continue;
        };

        let Some(name) = read_optional_string(&app_key, "DisplayName") else {
            continue;
        };

        let uninstall_command = read_optional_string(&app_key, "UninstallString")
            .or_else(|| read_optional_string(&app_key, "QuietUninstallString"));

        apps.push(InstalledApp {
            name,
            version: read_optional_string(&app_key, "DisplayVersion"),
            publisher: read_optional_string(&app_key, "Publisher"),
            install_date: read_optional_string(&app_key, "InstallDate"),
            has_uninstall_command: uninstall_command.is_some(),
        });
    }
}

#[tauri::command]
fn scan_installed_apps() -> Vec<InstalledApp> {
    let mut apps = Vec::new();

    collect_from_hive(
        RegKey::predef(HKEY_LOCAL_MACHINE),
        KEY_WOW64_64KEY,
        &mut apps,
    );
    collect_from_hive(
        RegKey::predef(HKEY_LOCAL_MACHINE),
        KEY_WOW64_32KEY,
        &mut apps,
    );
    collect_from_hive(
        RegKey::predef(HKEY_CURRENT_USER),
        KEY_WOW64_64KEY,
        &mut apps,
    );
    collect_from_hive(
        RegKey::predef(HKEY_CURRENT_USER),
        KEY_WOW64_32KEY,
        &mut apps,
    );

    let mut seen = HashSet::new();
    apps.retain(|app| {
        let fingerprint = format!(
            "{}|{}|{}",
            app.name.to_lowercase(),
            app.version.clone().unwrap_or_default().to_lowercase(),
            app.publisher.clone().unwrap_or_default().to_lowercase()
        );
        seen.insert(fingerprint)
    });

    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps
}

#[tauri::command]
fn check_winget() -> Result<String, String> {
    let output = Command::new("winget")
        .arg("--version")
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|error| format!("无法启动 winget：{error}"))?;

    if !output.status.success() {
        return Err("winget 当前不可用".to_string());
    }

    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(if version.is_empty() {
        "已安装".to_string()
    } else {
        version
    })
}

#[tauri::command]
fn install_app(package_id: String) -> InstallResult {
    if package_id.trim().is_empty() {
        return InstallResult {
            package_id,
            success: false,
            exit_code: None,
            message: "软件包 ID 不能为空".to_string(),
        };
    }

    let output = Command::new("winget")
        .args([
            "install",
            "--id",
            package_id.as_str(),
            "--exact",
            "--source",
            "winget",
            "--accept-source-agreements",
            "--accept-package-agreements",
            "--disable-interactivity",
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let message = if output.status.success() {
                if stdout.is_empty() {
                    "安装命令执行完成".to_string()
                } else {
                    stdout
                        .lines()
                        .last()
                        .unwrap_or("安装命令执行完成")
                        .to_string()
                }
            } else if !stderr.is_empty() {
                stderr.lines().last().unwrap_or("安装失败").to_string()
            } else if !stdout.is_empty() {
                stdout.lines().last().unwrap_or("安装失败").to_string()
            } else {
                "winget 返回失败状态".to_string()
            };

            InstallResult {
                package_id,
                success: output.status.success(),
                exit_code: output.status.code(),
                message,
            }
        }
        Err(error) => InstallResult {
            package_id,
            success: false,
            exit_code: None,
            message: format!("无法启动 winget：{error}"),
        },
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            check_winget,
            scan_installed_apps,
            install_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running PureInstall");
}
