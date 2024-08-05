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

use serde::Serialize;
use serde_json::{json, Value};
use std::sync::Arc;

use super::{KeyvError, Store, StoreError};

pub(super) const DEFAULT_NAMESPACE_NAME: &str = "localstore";

/// Key-Value Store Interface
///
/// Provides an synchronous interface to a key-value store. This implementation
/// allows for setting, getting, removing, and clearing key-value pairs in a
/// datastore with an optional Time-to-Live (TTL) for keys.
///
/// The `Keyv` struct is generic over any implementation of the `Store` trait,
/// thus can be backed by various storage engines.
///
/// # Examples
///
/// ## Create a new instance with in-memory store
///
/// ```
/// # use keyv::Keyv;
/// let keyv = Keyv::default();
/// ```
///
/// ## Set and get a value
///
/// ```
/// # use keyv::Keyv;
/// let keyv = Keyv::default();
///
/// keyv.set("array", vec!["hola", "test"]).unwrap();
///
/// match keyv.get("array").unwrap() {
///     Some(array) => {
///         let array: Vec<String> = serde_json::from_value(array).unwrap();
///         assert_eq!(array, vec!["hola".to_string(), "test".to_string()])
///     }
///     None => assert!(false),
/// }
///
/// keyv.set("string", "life long").unwrap();
/// match keyv.get("string").unwrap() {
///     Some(string) => {
///         let string: String = serde_json::from_value(string).unwrap();
///         assert_eq!(string, "life long");
///     }
///     None => assert!(false),
/// }
/// ```
pub struct Keyv {
    // store: Arc<Box<dyn Store>>,
    store: Arc<dyn Store>,
}

impl Keyv {
    /// Attempts to create a new `Keyv` instance with a custom store.
    ///
    /// This function will attempt to initialize the provided store. If the initialization
    /// is successful, a new `Keyv` instance is returned.
    ///
    /// # Arguments
    ///
    /// * `store` - A custom store implementing the `Store` trait.
    ///
    /// # Errors
    ///
    /// Returns `KeyvError` if the store fails to initialize.
    ///
    /// # Examples
    ///
    /// ```
    /// # use keyv::{Keyv};
    /// # use keyv::{KeyvStoreBuilder};
    ///
    /// let store = KeyvStoreBuilder::new()
    ///     .uri("sqlite::memory:")
    ///     .table_name("custom_table_name")
    ///     .build()
    ///     .unwrap();
    ///
    /// let keyv = Keyv::try_new(store).unwrap();
    /// ```
    pub async fn try_new<S: Store + 'static>(store: S) -> Result<Self, KeyvError> {
        store.initialize().await?;
        Ok(Self { store: Arc::new(store) })
    }

    /// Sets a value for a given key without a TTL.
    ///
    /// # Arguments
    ///
    /// * `key` - The key under which the value is stored.
    /// * `value` - The value to store. Must implement `Serialize`.
    ///
    /// # Errors
    ///
    /// Returns `KeyvError` if the operation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # use keyv::Keyv;
    /// let keyv = Keyv::default();
    /// keyv.set("key", "hello world").unwrap();
    /// ```
    pub async fn set<T: Serialize>(&self, key: &str, value: T) -> Result<(), KeyvError> {
        let json_value = serde_json::to_value(value).map_err(|e| StoreError::SerializationError { source: e })?;
        self.store.set(key, json_value, None).await?;
        Ok(())
    }

    /// Sets a value for a given key with an expiry TTL (Time-To-Live).
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice that holds the key.
    /// * `value` - The value to be stored, which must implement `Serialize`.
    /// * `ttl` - The time-to-live (in seconds) for the key-value pair.
    ///
    /// # Returns
    ///
    /// Returns an `Ok` result on successful insertion, or a `KeyvError` on failure.
    ///
    /// # Examples
    ///
    /// ```
    /// # use keyv::Keyv;
    /// let keyv = Keyv::default();
    /// keyv.set_with_ttl("temp_key", "temp_value", 3600).unwrap(); // Expires in 1 hour
    /// ```
    pub async fn set_with_ttl<T: Serialize>(&self, key: &str, value: T, ttl: u64) -> Result<(), KeyvError> {
        Ok(self.store.set(key, json!(value), Some(ttl)).await?)
    }

    /// Retrieves a value based on a key.
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice that holds the key to retrieve the value for.
    ///
    /// # Returns
    ///
    /// Returns an `Ok` result with `Option<Value>` on success, where `None` indicates the
    /// key does not exist, or a `KeyvError` on failure.
    ///
    /// # Examples
    ///
    /// ```
    /// # use keyv::Keyv;
    /// let keyv = Keyv::default();
    ///
    /// keyv.set("array", vec!["hola", "test"]).unwrap();
    ///
    /// match keyv.get("array").unwrap() {
    ///     Some(array) => {
    ///         let array: Vec<String> = serde_json::from_value(array).unwrap();
    ///         assert_eq!(array, vec!["hola".to_string(), "test".to_string()])
    ///     }
    ///     None => assert!(false),
    /// }
    ///
    /// keyv.set("string", "life long").unwrap();
    /// match keyv.get("string").unwrap() {
    ///     Some(string) => {
    ///         let string: String = serde_json::from_value(string).unwrap();
    ///         assert_eq!(string, "life long");
    ///     }
    ///     None => assert!(false),
    /// }
    /// ```
    pub async fn get(&self, key: &str) -> Result<Option<Value>, KeyvError> {
        Ok(self.store.get(key).await?)
    }

    /// Removes a specified key from the store.
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice that represents the key to be removed.
    ///
    /// # Returns
    ///
    /// Returns an `Ok` result if the key has been successfully removed, or a `KeyvError`
    /// on failure.
    ///
    /// # Examples
    ///
    /// ```
    /// # use keyv::Keyv;
    /// let keyv = Keyv::default();
    /// keyv.remove("my_key").unwrap(); // Removes "my_key" from the store
    /// ```
    pub async fn remove(&self, key: &str) -> Result<(), KeyvError> {
        Ok(self.store.remove(key).await?)
    }

    /// Removes multiple keys from the store in one operation.
    ///
    /// # Arguments
    ///
    /// * `keys` - A slice of strings or string-like objects that represent the keys to be removed.
    ///
    /// # Returns
    ///
    /// Returns an `Ok` result if the keys have been successfully removed, or a `KeyvError`
    /// on failure.
    ///
    /// # Examples
    ///
    /// ```
    /// # use keyv::Keyv;
    /// let keyv = Keyv::default();
    /// keyv.remove_many(&["key1", "key2"]).unwrap(); // Removes "key1" and "key2"
    /// ```
    pub async fn remove_many<T: AsRef<str> + Sync>(&self, keys: &[T]) -> Result<(), KeyvError> {
        let keys: Vec<&str> = keys.iter().map(|k| k.as_ref()).collect();
        Ok(self.store.remove_many(&keys).await?)
    }

    /// Clears the entire store, removing all key-value pairs.
    ///
    /// # Returns
    ///
    /// Returns an `Ok` result if the store has been successfully cleared, or a `KeyvError`
    /// on failure.
    ///
    /// # Examples
    ///
    /// ```
    /// # use keyv::Keyv;
    /// let keyv = Keyv::default();
    /// keyv.clear().unwrap(); // Clears the entire store
    /// ```
    pub async fn clear(&self) -> Result<(), KeyvError> {
        Ok(self.store.clear().await?)
    }
}

// FIXME : implement default store
// impl Default for Keyv {
//     fn default() -> Self {
//         Self {
//             store: Arc::new(Connection::open("sqlite::memory:")),
//         }
//     }
// }
