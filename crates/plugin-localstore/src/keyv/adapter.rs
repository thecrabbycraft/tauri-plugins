// Copyright © 2024 Crabby Craft - All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/*!
 * Portions of this file are based on code from `chrisllontop/keyv-rust`.
 * MIT Licensed, Copyright (c) 2023 Christian Llontop.
 *
 * Credits to Alexandru Bereghici: https://github.com/chrisllontop/keyv-rust
 */

use libsql::{params, Builder, Connection};
use serde_json::Value;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;

use super::DEFAULT_NAMESPACE_NAME;
use super::{Store, StoreError};

/// Builder for creating a `KeyvStore`.
///
/// This builder allows for configuring a `KeyvStore` with custom
/// settings such as a specific database file URI and a table name.
/// It provides a flexible way to initialize the store depending on the
/// application's requirements.
///
/// # Examples
///
/// ## Initializing with a Database File URI
///
/// ```rust,no_run
/// # use keyv::{KeyvStoreBuilder};
/// # #[tokio::main]
/// # async fn main(){
/// let store = KeyvStoreBuilder::new()
///     .uri("sqlite::memory:")
///     .table_name("custom_table_name")
///     .build()
///     .unwrap();
///  }
/// ```
///
/// ## Using an Existing Connection Pool
///
/// ```rust,no_run
/// # use std::sync::Arc;
/// # use keyv::{KeyvStoreBuilder};
/// # use libsql::Connection;
///
/// # #[tokio::main]
/// # async fn main() {
/// let conn: Arc<Connnection> = Arc::new(Connection::open(path).unwrap());
///
/// let store = KeyvStoreBuilder::new()
///     .connnection(conn)
///     .table_name("custom_table_name")
///     .build()
///     .unwrap();
/// }
/// ```
pub struct KeyvStoreBuilder {
    uri: Option<PathBuf>,
    token: Option<String>,
    connnection: Option<Arc<Connection>>,
    table_name: Option<String>,
}

impl KeyvStoreBuilder {
    pub fn new() -> Self {
        Self {
            uri: None,
            token: None,
            connnection: None,
            table_name: None,
        }
    }

    /// Sets the table name for the `KeyvStore`.
    ///
    /// This method configures the table name to be used by the store. If not set,
    /// `DEFAULT_TABLE_NAME` from the configuration will be used.
    pub fn table_name<S: Into<String>>(mut self, table: S) -> Self {
        self.table_name = Some(table.into());
        self
    }

    /// Sets the database URI for connecting to the SQLite database.
    ///
    /// This method configures the database URI. It's required if no existing connection is provided.
    pub fn uri<S: Into<PathBuf>>(mut self, uri: S) -> Self {
        self.uri = Some(uri.into());
        self
    }

    /// Sets the database token for authentication with the database.
    ///
    /// This method configures the database token. It's required if using authentication.
    pub fn token<S: Into<String>>(mut self, token: S) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Uses an existing connection for the `KeyvStore`.
    ///
    /// This method allows for using an already configured `Pool`. If set,
    /// the `uri` option is ignored.
    pub fn connnection(mut self, connnection: Arc<Connection>) -> Self {
        self.connnection = Some(connnection);
        self
    }

    /// Builds the `KeyvStore` based on the provided configurations.
    ///
    /// Finalizes the builder and creates an `KeyvStore` instance.
    /// It requires either a database URI or an existing connection to be set.
    ///
    /// # Returns
    /// This method returns a `Result` which, on success, contains the initialized `KeyvStore`.
    /// On failure, it returns a `StoreError` indicating what went wrong during the initialization.
    pub async fn build(self) -> Result<KeyvStore, StoreError> {
        let connnection = match self.connnection {
            Some(connnection) => connnection,
            None => {
                let path = self
                    .uri
                    .expect("KeyvStore requires either a URI or an existing connnection to be set");

                // If the token is set, use the remote database connection.
                let db = if let Some(token) = self.token {
                    Builder::new_remote(path.display().to_string(), token)
                        .build()
                        .await
                        .map_err(|_| StoreError::ConnectionError("Failed to create database connection".to_string()))?
                } else {
                    Builder::new_local(path)
                        .build()
                        .await
                        .map_err(|_| StoreError::ConnectionError("Failed to create database connection".to_string()))?
                };

                let conn = db
                    .connect()
                    .map_err(|_| StoreError::ConnectionError("Failed to create database connnection".to_string()))?;

                Arc::new(conn)
            }
        };

        let table_name = self.table_name.unwrap_or_else(|| {
            log::warn!("Table name not set, using default table name");
            DEFAULT_NAMESPACE_NAME.to_string()
        });

        Ok(KeyvStore {
            connnection,
            table_name,
        })
    }
}

pub struct KeyvStore {
    pub(crate) connnection: Arc<Connection>,
    pub(crate) table_name: String,
}

impl KeyvStore {
    fn get_table_name(&self) -> String {
        self.table_name.clone()
    }
}

impl Store for KeyvStore {
    fn initialize(&self) -> Pin<Box<dyn Future<Output = Result<(), StoreError>> + Send + '_>> {
        let query = format!(
            r#"
                CREATE TABLE IF NOT EXISTS {table_name} (
                    key TEXT PRIMARY KEY,
                    value TEXT NOT NULL,
                    updated_at TEXT DEFAULT (datetime('now', 'localtime')),
                    UNIQUE(key)
                ) STRICT;
                CREATE INDEX IF NOT EXISTS {table_name}_key_idx ON {table_name} (key);
                CREATE TRIGGER IF NOT EXISTS {table_name}_update_trigger
                AFTER UPDATE ON {table_name}
                BEGIN
                    UPDATE {table_name} SET updated_at = datetime('now', 'localtime') WHERE key = NEW.key;
                END;
            "#,
            table_name = self.get_table_name()
        );

        let conn = &*self.connnection;

        Box::pin(async move {
            conn.execute_batch(&query)
                .await
                .map_err(|e| StoreError::QueryError(format!("Failed to initialize the database table: {}", e)))?;

            Ok(())
        })
    }

    fn get(&self, key: &str) -> Pin<Box<dyn Future<Output = Result<Option<Value>, StoreError>> + Send + '_>> {
        let query = format!("SELECT value FROM {} WHERE key = ?1 LIMIT 1", self.get_table_name());

        let conn = &*self.connnection;
        let key = key.to_string();

        Box::pin(async move {
            let start = Instant::now();

            let mut stmt = conn
                .prepare(&query)
                .await
                .map_err(|e| StoreError::QueryError(format!("Failed to set the statement: {:?}", e)))?;

            let result = stmt
                .query_row(params![key.clone()])
                .await
                .map_err(|e| StoreError::QueryError(format!("Failed to fetch the value: {:?}", e)))?;

            let row_value: String = result
                .get(0)
                .map_err(|e| StoreError::QueryError(format!("Failed to get the value: {:?}", e)))?;

            let value = serde_json::to_value(row_value).map_err(|e| StoreError::SerializationError { source: e })?;

            let duration = start.elapsed();
            log::debug!("Keyv store get: {:?} | {} | {:?}", duration, key, value);

            Ok(Some(value))
        })
    }

    fn set(
        &self,
        key: &str,
        value: Value,
        _ttl: Option<u64>,
    ) -> Pin<Box<dyn Future<Output = Result<(), StoreError>> + Send + '_>> {
        let query = format!(
            "INSERT INTO {} (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = EXCLUDED.value",
            self.get_table_name()
        );

        let conn = &*self.connnection;
        let key = key.to_string();

        Box::pin(async move {
            let start = Instant::now();

            let value_str = match value {
                Value::String(ref s) => s.clone(), // If the value is a string, use the original string.
                Value::Number(ref n) => n.to_string(), // If the value is a number, use the number string representation.
                Value::Null => "".to_string(),         // If value is null, use the empty string.
                _ => value.to_string(),                // If the value is an object or other type, serialize it as JSON.
            };

            let mut stmt = conn
                .prepare(&query)
                .await
                .map_err(|_| StoreError::QueryError("Failed to set the statement".to_string()))?;

            stmt.execute(params![key.clone(), value_str.clone()])
                .await
                .map_err(|_| StoreError::QueryError("Failed to set the value".to_string()))?;

            let duration = start.elapsed();
            log::debug!("Keyv store set: {:?} | {} | {}", duration, key, value_str);

            Ok(())
        })
    }

    fn remove(&self, key: &str) -> Pin<Box<dyn Future<Output = Result<(), StoreError>> + Send + '_>> {
        let query = format!("DELETE FROM {} WHERE key = ?1", self.get_table_name());

        let conn = &*self.connnection;

        let key = key.to_string();

        Box::pin(async move {
            let start = Instant::now();

            let mut stmt = conn
                .prepare(&query)
                .await
                .map_err(|_| StoreError::QueryError("Failed to set the statement".to_string()))?;

            stmt.execute(params![key.clone()])
                .await
                .map_err(|_| StoreError::QueryError("Failed to remove the key".to_string()))?;

            let duration = start.elapsed();
            log::debug!("Keyv store remove: {:?} | {}", duration, key);

            Ok(())
        })
    }

    fn remove_many(&self, _keys: &[&str]) -> Pin<Box<dyn Future<Output = Result<(), StoreError>> + Send + '_>> {
        // let query = format!(
        //     "DELETE FROM {} WHERE key IN ({})",
        //     self.get_table_name(),
        //     keys.iter().map(|_| "?").collect::<Vec<&str>>().join(",")
        // );

        // let mut query = sqlx::query(&query);
        // for key in keys {
        //     query = query.bind(key);
        // }

        Box::pin(async move {
            let start = Instant::now();

            // query
            //     .execute(&*self.connnection)
            //
            //     .map_err(|e| StoreError::QueryError(format!("Failed to remove the keys: {}", e.to_string())))?;

            // let conn = &*self.connnection;

            let duration = start.elapsed();
            log::debug!("Keyv store remove_many: {:?}", duration);

            Ok(())
        })
    }

    fn clear(&self) -> Pin<Box<dyn Future<Output = Result<(), StoreError>> + Send + '_>> {
        let query = format!("DELETE FROM {}", self.get_table_name());

        let conn = &*self.connnection;

        Box::pin(async move {
            conn.execute(&query, params![])
                .await
                .map_err(|_| StoreError::QueryError("Failed to clear the table".to_string()))?;

            Ok(())
        })
    }
}
