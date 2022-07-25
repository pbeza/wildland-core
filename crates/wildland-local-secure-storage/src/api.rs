use crate::LSSResult;
#[cfg(test)]
use mockall::automock;
use std::fmt::Debug;

#[cfg_attr(test, automock)]
pub trait LocalSecureStorage: Debug {
    /// Inserts a key-value pair into the LSS.
    /// If the map did not have this key present, None is returned.
    /// If the map did have this key present, the value is updated, and the old value is returned.
    fn insert(&mut self, key: String, value: Vec<u8>) -> LSSResult<Option<Vec<u8>>>;

    /// Returns a copy of the value corresponding to the key.
    fn get(&self, key: String) -> LSSResult<Option<Vec<u8>>>;

    /// Returns true if the map contains a value for the specified key.
    fn contains_key(&self, key: String) -> LSSResult<bool>;

    /// Returns all keys in arbitrary order.
    fn keys(&self) -> LSSResult<Vec<String>>;

    /// Removes a key from the map, returning the value at the key if the key was previously in the map.
    fn remove(&mut self, key: String) -> LSSResult<Option<Vec<u8>>>;

    /// Returns the number of elements in the map.
    fn len(&self) -> LSSResult<usize>;

    /// Returns true if the map contains no elements, false otherwise.
    fn is_empty(&self) -> LSSResult<bool>;
}
