// Copyright © 2024 Crabby Craft - All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// @ref: https://github.com/RandomEngy/tauri-sqlite

// use rusqlite::Connection;
// use rusqlite_migration::{Migrations, M};
// use tauri::{AppHandle, Runtime};

// use super::{Keyv, KeyvStoreBuilder};
// use crate::get_db_path;

// /// Initializes the database connection, creating the database file if needed,
// /// and running the database migrations if it's out of date.
// pub fn initialize<R: Runtime>(
//     app_handle: &AppHandle<R>,
//     file_name: Option<String>,
//     migrations: Vec<M>,
// ) -> Result<Connection, rusqlite::Error> {
//     let db_file_path = get_db_path(app_handle, file_name)
//         .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(tauri::Error::Anyhow(e.into()))))?;

//     let mut db = Connection::open(db_file_path)?;

//     // Define the migration queries
//     // let migrations: Vec<M> = vec![
//     //     M::up(include_str!("../../migrations/0001_initialize_tables.sql")),
//     //     // M::up(include_str!("../../migrations/0002_foundation_tables.sql")),
//     // ];

//     let migrate = Migrations::new(migrations);

//     // Apply some PRAGMA, often better to do it outside of migrations
//     db.pragma_update_and_check(None, "journal_mode", &"WAL", |_| Ok(()))?;

//     // Update the database schema, atomically
//     migrate.to_latest(&mut db).expect("Failed to migrate database");

//     Ok(db)
// }
