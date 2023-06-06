//! Operations for Key-Value storage

use std::collections::HashMap;

use super::*;

/// Generic Key-Value storage API
pub trait SeralizedStore {
    /// Error from using the store
    type Error;

    /// Stores a key-value pair
    /// Returns previous value if one existed
    fn insert<Key, Value>(&mut self, key: Key, value: Value) -> Result<Option<Value>, Self::Error>
    where
        Key: serde::Serialize,
        Value: serde::de::DeserializeOwned + serde::Serialize;

    /// Gets the value stored under key or Err(None)
    fn get<Key, Value>(&self, key: &Key) -> Result<Option<Value>, Self::Error>
    where
        Key: serde::Serialize,
        Value: serde::de::DeserializeOwned + serde::Serialize;

    /// Clears the value stored under key
    /// Returns the value stored under key or Err(None)
    fn remove<Key, Value>(&mut self, key: &Key) -> Result<Option<Value>, Self::Error>
    where
        Key: serde::Serialize,
        Value: serde::de::DeserializeOwned + serde::Serialize;
}

/// Errors for Stores that implement the [Web Storage API](https://developer.mozilla.org/en-US/docs/Web/API/Storage).
/// Since the backing type is a String for Keys and Values, to facilitate storing generic types serialisation is used.
#[derive(Debug)]
pub enum WebStoreError {
    /// [A security error](https://developer.mozilla.org/en-US/docs/Web/API/Window/sessionStorage#exceptions) may
    /// occur if accessing the store is prohibited or insecure.
    AccessDenied,
    /// Occurs if the store is full, see more [here](https://developer.mozilla.org/en-US/docs/Web/API/Storage/setItem#exceptions).
    StorageFull,
    /// Errors serialising generic types into or from store.
    Serialisation(serde_json::Error),
    /// Undocumented Store Error
    Unknown,
}
impl std::fmt::Display for WebStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl PartialEq for WebStoreError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (WebStoreError::AccessDenied, WebStoreError::AccessDenied) => true,
            (WebStoreError::StorageFull, WebStoreError::StorageFull) => true,
            (WebStoreError::Serialisation(s), WebStoreError::Serialisation(o))
                if s.to_string() == o.to_string() =>
            {
                true
            }
            (WebStoreError::Unknown, WebStoreError::Unknown) => true,
            _ => false,
        }
    }
}
impl std::error::Error for WebStoreError {}

trait WebStoreGetter<Key, Value>
where
    Key: serde::Serialize,
    Value: serde::Serialize + serde::de::DeserializeOwned,
{
    fn store(&self) -> Result<web_sys::Storage, WebStoreError>;
}

macro_rules! web_store {
    ($web_store_function:ident, $web_store_type:ident) => {
        /// Phantom Type for the [web_sys::Window::$web_store_function] getter
        #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
        pub struct $web_store_type {}

        impl $web_store_type {
            /// Creates a [$web_store_type] reference
            pub fn new() -> Self {
                Self {}
            }

            fn store(&self) -> Result<web_sys::Storage, WebStoreError> {
                browser_window()
                    .$web_store_function()
                    .map_err(|_| WebStoreError::AccessDenied)?
                    .ok_or(WebStoreError::Unknown)
            }
        }

        impl SeralizedStore for $web_store_type {
            type Error = WebStoreError;

            fn insert<Key, Value>(
                &mut self,
                key: Key,
                value: Value,
            ) -> Result<Option<Value>, WebStoreError>
            where
                Key: serde::Serialize,
                Value: serde::de::DeserializeOwned + serde::Serialize,
            {
                let key = serde_json::to_string(&key).map_err(WebStoreError::Serialisation)?;
                self.store().and_then(|storage| {
                    let ret = match storage.get_item(&key).map_err(|_| WebStoreError::Unknown)? {
                        Some(value) => {
                            serde_json::from_str(&value).map_err(WebStoreError::Serialisation)?
                        }
                        None => None,
                    };
                    storage
                        .set_item(
                            &key,
                            &serde_json::to_string(&value).map_err(WebStoreError::Serialisation)?,
                        )
                        .map_err(|_| WebStoreError::StorageFull)?;
                    Ok(ret)
                })
            }

            fn get<Key, Value>(&self, key: &Key) -> Result<Option<Value>, WebStoreError>
            where
                Key: serde::Serialize,
                Value: serde::de::DeserializeOwned + serde::Serialize,
            {
                match self
                    .store()?
                    .get_item(&serde_json::to_string(&key).map_err(WebStoreError::Serialisation)?)
                    .map_err(|_| WebStoreError::Unknown)
                {
                    Ok(Some(value)) => Ok(Some(
                        serde_json::from_str(&value).map_err(WebStoreError::Serialisation)?,
                    )),
                    Ok(None) => Ok(None),
                    Err(err) => Err(err),
                }
            }

            fn remove<Key: serde::Serialize, Value: serde::de::DeserializeOwned>(
                &mut self,
                key: &Key,
            ) -> Result<Option<Value>, WebStoreError>
            where
                Key: serde::Serialize,
                Value: serde::de::DeserializeOwned + serde::Serialize,
            {
                let key = serde_json::to_string(&key).map_err(WebStoreError::Serialisation)?;
                self.store().and_then(|storage| {
                    let ret = match storage.get_item(&key).map_err(|_| WebStoreError::Unknown)? {
                        Some(value) => {
                            serde_json::from_str(&value).map_err(WebStoreError::Serialisation)?
                        }
                        None => None,
                    };
                    storage
                        .remove_item(&key)
                        .map_err(|_| WebStoreError::Unknown)?;
                    Ok(ret)
                })
            }
        }
    };
}

web_store! {session_storage, SessionStore}
web_store! {local_storage, LocalStore}

impl SeralizedStore for HashMap<String, String> {
    type Error = serde_json::Error;

    fn insert<Key, Value>(&mut self, key: Key, value: Value) -> Result<Option<Value>, Self::Error>
    where
        Key: serde::Serialize,
        Value: serde::de::DeserializeOwned + serde::Serialize,
    {
        match self.insert(serde_json::to_string(&key)?, serde_json::to_string(&value)?) {
            Some(v) => serde_json::from_str(&v),
            None => Ok(None),
        }
    }

    fn get<Key, Value>(&self, key: &Key) -> Result<Option<Value>, Self::Error>
    where
        Key: serde::Serialize,
        Value: serde::de::DeserializeOwned + serde::Serialize,
    {
        match self.get(&serde_json::to_string(&key)?) {
            Some(v) => serde_json::from_str(v),
            None => Ok(None),
        }
    }

    fn remove<Key, Value>(&mut self, key: &Key) -> Result<Option<Value>, Self::Error>
    where
        Key: serde::Serialize,
        Value: serde::de::DeserializeOwned + serde::Serialize,
    {
        match self.remove(&serde_json::to_string(&key)?) {
            Some(v) => serde_json::from_str(&v),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    type WebStoreResult<Value> = Result<Option<Value>, WebStoreError>;

    #[wasm_bindgen_test]
    fn test_web_store_serialise() {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct StructType {
            number: i32,
            string: String,
        }

        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        enum EnumType {
            Field,
        }

        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct TupleType(f64);

        let mut store = SessionStore::new();

        assert_eq!(
            store.insert(
                "struct",
                StructType {
                    number: 69,
                    string: "String".to_string()
                }
            ),
            Ok(None)
        );
        assert_eq!(store.insert("enum", EnumType::Field), Ok(None));
        assert_eq!(store.insert("tuple", TupleType(69.0)), Ok(None));
        assert_eq!(store.insert("string", "String".to_string()), Ok(None));
        assert_eq!(store.insert("number", 69), Ok(None));

        assert_eq!(
            store.get(&"struct"),
            Ok(Some(StructType {
                number: 69,
                string: "String".to_string()
            }))
        );
        assert_eq!(store.get(&"enum"), Ok(Some(EnumType::Field)));
        assert_eq!(store.get(&"tuple"), Ok(Some(TupleType(69.0))));
        assert_eq!(store.get(&"string"), Ok(Some("String".to_string())));
        assert_eq!(store.get(&"number"), Ok(Some(69)));

        assert!(matches!(
            store.remove(&"struct") as WebStoreResult<EnumType>,
            Err(WebStoreError::Serialisation(_))
        ));
        assert!(matches!(
            store.remove(&"struct") as WebStoreResult<TupleType>,
            Err(WebStoreError::Serialisation(_))
        ));
        assert!(matches!(
            store.remove(&"struct") as WebStoreResult<String>,
            Err(WebStoreError::Serialisation(_))
        ));
        assert!(matches!(
            store.remove(&"struct") as WebStoreResult<i32>,
            Err(WebStoreError::Serialisation(_))
        ));

        assert_eq!(
            store.get(&"struct"),
            Ok(Some(StructType {
                number: 69,
                string: "String".to_string()
            }))
        );
        assert_eq!(store.get(&"enum"), Ok(Some(EnumType::Field)));
        assert_eq!(store.get(&"tuple"), Ok(Some(TupleType(69.0))));
        assert_eq!(store.get(&"string"), Ok(Some("String".to_string())));
        assert_eq!(store.get(&"number"), Ok(Some(69)));

        assert_eq!(
            store.remove(&"struct"),
            Ok(Some(StructType {
                number: 69,
                string: "String".to_string()
            }))
        );
        assert_eq!(store.remove(&"enum"), Ok(Some(EnumType::Field)));
        assert_eq!(store.remove(&"tuple"), Ok(Some(TupleType(69.0))));
        assert_eq!(store.remove(&"string"), Ok(Some("String".to_string())));
        assert_eq!(store.remove(&"number"), Ok(Some(69)));

        assert_eq!(store.get(&"struct") as WebStoreResult<StructType>, Ok(None));
        assert_eq!(store.get(&"enum") as WebStoreResult<EnumType>, Ok(None));
        assert_eq!(store.get(&"tuple") as WebStoreResult<TupleType>, Ok(None));
        assert_eq!(store.get(&"string") as WebStoreResult<String>, Ok(None));
        assert_eq!(store.get(&"number") as WebStoreResult<i32>, Ok(None));
    }
}
