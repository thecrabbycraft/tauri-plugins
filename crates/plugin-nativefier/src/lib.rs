// Copyright © 2024 Crabby Craft - All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[cfg(target_os = "macos")]
#[macro_use]
extern crate cocoa;

#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;

mod cmd;

pub mod theme;
pub mod utils;

use tauri::plugin::Builder as PluginBuilder;
use tauri::plugin::TauriPlugin;
use tauri::Runtime;

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    PluginBuilder::new("nativefier")
        .setup(|_app, _api| Ok(()))
        .invoke_handler(tauri::generate_handler![
            cmd::list_font_mono,
            cmd::list_font_sans,
            cmd::open_data_directory,
            cmd::open_in_browser,
            cmd::open_log_directory,
            cmd::open_log_file,
            cmd::toggle_devtools,
            theme::get_theme,
            theme::set_theme,
        ])
        .on_navigation(|window, url| {
            log::debug!("navigation {} {url}", window.label());
            true
        })
        .build()
}
