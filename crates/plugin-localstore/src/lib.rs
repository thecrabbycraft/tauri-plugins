// Copyright © 2024 Crabby Craft - All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

mod cmd;

use tauri::plugin::{Builder, TauriPlugin};
use tauri::Runtime;

use cmd::toggle_devtools;

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("localstore")
        .setup(|_app, _api| Ok(()))
        .invoke_handler(tauri::generate_handler![toggle_devtools])
        .on_navigation(|window, url| {
            log::debug!("navigation {} {url}", window.label());
            true
        })
        .build()
}
