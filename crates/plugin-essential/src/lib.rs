mod cmd;

use tauri::plugin::{Builder, TauriPlugin};
use tauri::Runtime;

use cmd::toggle_devtools;

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("essential")
        .setup(|_app, _api| Ok(()))
        .invoke_handler(tauri::generate_handler![toggle_devtools])
        .on_navigation(|window, url| {
            log::debug!("navigation {} {url}", window.label());
            true
        })
        .build()
}
