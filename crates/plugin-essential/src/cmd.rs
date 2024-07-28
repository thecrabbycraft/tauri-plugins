use tauri::{Runtime, WebviewWindow};

#[tauri::command(rename_all = "snake_case")]
pub fn toggle_devtools<R: Runtime>(window: WebviewWindow<R>) {
    if !window.is_devtools_open() {
        window.open_devtools()
    } else if window.is_devtools_open() {
        window.close_devtools()
    }
}
