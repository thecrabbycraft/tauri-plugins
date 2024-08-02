// Copyright © 2024 Crabby Craft - All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

mod cmd;

mod builder;
pub use builder::*;

mod adapter;
pub use adapter::*;

mod errors;
pub use errors::*;

mod keyv;
pub use keyv::*;

mod store;
pub use store::*;

// Re-export rusqlite_migration
pub use rusqlite_migration::{Migrations, M};

use tauri::plugin::{Builder, TauriPlugin};
use tauri::Runtime;

pub const DEFAULT_NAMESPACE_NAME: &str = "localstore";

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("localstore")
        .setup(|_app, _api| Ok(()))
        .invoke_handler(tauri::generate_handler![])
        .on_navigation(|window, url| {
            log::debug!("navigation {} {url}", window.label());
            true
        })
        .build()
}
