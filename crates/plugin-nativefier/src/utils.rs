use anyhow::anyhow;
use font_kit::source::SystemSource;
use once_cell::sync::Lazy;
use serde_json::json;
use serde_json::value::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Mutex;
use url::Url;
use webbrowser::{self, Browser, BrowserOptions};

use tauri::{path::BaseDirectory, AppHandle, Manager, Runtime, WebviewWindow};
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
use tauri_plugin_shell::{open, ShellExt};

static SANS_FONT_CACHE: Lazy<Mutex<Option<Vec<JsonValue>>>> = Lazy::new(|| Mutex::new(None));
static MONO_FONT_CACHE: Lazy<Mutex<Option<Vec<JsonValue>>>> = Lazy::new(|| Mutex::new(None));

pub fn navigate_to<R: Runtime>(window: &mut WebviewWindow<R>, url: &str) -> Result<(), String> {
    let current_url = window.url().map_err(|e| e.to_string())?;
    let new_url = Url::join(&current_url, url).map_err(|e| e.to_string())?;

    // Handle if current url is the same as the new url
    if current_url == new_url {
        return Ok(());
    }

    log::debug!("Navigating to: {}", new_url.as_str());

    let _ = window.navigate(new_url);

    Ok(())
}

pub fn force_reload<R: Runtime>(window: &mut WebviewWindow<R>) -> Result<(), String> {
    let current_url = window.url().map_err(|e| e.to_string())?;

    log::debug!("Force reloading page: {}", current_url.as_str());

    let _ = window.navigate(current_url);

    Ok(())
}

pub fn open_in_browser(url: &str) {
    let browser = Browser::Default;
    let opts = BrowserOptions::default();
    if webbrowser::open_browser_with_options(browser, url, &opts).is_ok() {
        log::debug!("Opened {} in the default browser", url);
    }
}

pub fn open_with_shell(url: &str) {
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()
            .expect("failed to open shell");
    }

    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .arg("/C")
            .arg("start")
            .arg(url)
            .spawn()
            .expect("failed to open shell");
    }

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg(url)
            .spawn()
            .expect("failed to open shell");
    }
}

pub fn open_with<R: Runtime>(app: &AppHandle<R>, url: String, with: Option<open::Program>) -> Result<(), String> {
    // Use the AppHandle to access the shell API and open the URL
    app.shell().open(&url, with).map_err(|e| e.to_string())
}

pub fn open_data_directory<R: Runtime>(app: &AppHandle<R>) {
    let log_file_path = app
        .path()
        .resolve("", tauri::path::BaseDirectory::AppData)
        .expect("failed to get log file path");

    let log_path_str = log_file_path.display().to_string();

    if !log_file_path.exists() {
        log::warn!("Log directory not found: {log_path_str}");
        let _ = app
            .dialog()
            .message(format!("Directory {} not found!", log_path_str))
            .kind(MessageDialogKind::Error)
            .title("Cannot Open Log Directory")
            .blocking_show();
        return;
    }

    open_with_shell(log_path_str.as_str());
}

pub fn open_log_directory<R: Runtime>(app: &AppHandle<R>) {
    let log_file_path = app
        .path()
        .resolve("", BaseDirectory::AppLog)
        .expect("failed to get log file path");

    let log_path_str = log_file_path.display().to_string();

    if !log_file_path.exists() {
        log::warn!("Log directory not found: {log_path_str}");
        let _ = app
            .dialog()
            .message(format!("Directory {} not found!", log_path_str))
            .kind(MessageDialogKind::Error)
            .title("Cannot Open Log Directory")
            .blocking_show();
        return;
    }

    open_with_shell(log_path_str.as_str());
}

pub fn open_log_file<R: Runtime>(app: &AppHandle<R>, file_name: &str) {
    let log_file_path = app
        .path()
        .resolve(file_name, BaseDirectory::AppLog)
        .expect("failed to get log file path");

    let log_file_str = format!("{}.log", log_file_path.display());

    if !log_file_str.ends_with(".log") || !std::path::Path::new(&log_file_str).exists() {
        log::warn!("Log file not found: {}", log_file_str);
        app.dialog()
            .message(format!("Log file not found: {}", log_file_str))
            .kind(MessageDialogKind::Error)
            .title("Cannot Open Log File")
            .blocking_show();
        return;
    }

    open_with_shell(&log_file_str);
}

/// Utility function to list fonts based on the monospace filter.
/// # Arguments
/// * `monospace_filter` - A boolean that specifies if only monospace fonts should be listed.
pub fn list_fonts(monospace_filter: bool) -> tauri::Result<Vec<JsonValue>> {
    let cache = if monospace_filter {
        &MONO_FONT_CACHE
    } else {
        &SANS_FONT_CACHE
    };

    let mut cache = cache
        .lock()
        .map_err(|e| tauri::Error::Anyhow(anyhow!("Failed to lock cache: {}", e)))?;

    if let Some(cached_fonts) = cache.as_ref() {
        return Ok(cached_fonts.clone());
    }

    let source = SystemSource::new();
    let fonts = source.all_fonts().map_err(|e| tauri::Error::Anyhow(e.into()))?;

    let mut result_set = HashMap::new();

    for font in fonts {
        if let Ok(font) = font.load() {
            if font.is_monospace() == monospace_filter {
                let family_name = font.family_name().to_string();
                let display_name = font.full_name().to_string();
                let style = font.properties().style.to_string();
                let id = family_name.to_lowercase().replace(' ', "-");

                result_set.entry(family_name.clone()).or_insert_with(|| {
                    json!({
                        "id": id,
                        "displayName": display_name,
                        "family": family_name,
                        "style": style
                    })
                });
            }
        }
    }

    let mut result: Vec<JsonValue> = result_set.into_values().collect();

    result.sort_by(|a, b| {
        a["family"]
            .as_str()
            .unwrap_or("")
            .cmp(b["family"].as_str().unwrap_or(""))
    });

    *cache = Some(result.clone());

    Ok(result)
}
