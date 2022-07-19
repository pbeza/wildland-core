#[cfg(test)]
use mockall::automock;
use crate::LSSResult;

#[cfg_attr(test, automock)]
pub trait LocalSecureStorage {
    /// Inserts a key-value pair into the LSS.
    /// If the map did not have this key present, None is returned.
    /// If the map did have this key present, the value is updated, and the old value is returned.
    fn insert(&mut self, key: String, value: Vec<u8>) -> LSSResult<Option<Vec<u8>>>;

    /// Returns a copy of the value corresponding to the key.
    fn get(&self, key: String) -> Option<Vec<u8>>;

    /// Returns true if the map contains a value for the specified key.
    fn contains_key(&self, key: String) -> bool;

    /// Returns all keys in arbitrary order.
    fn keys(&self) -> Vec<String>;

    /// Removes a key from the map, returning the value at the key if the key was previously in the map.
    fn remove(&mut self, key: String) -> LSSResult<Option<Vec<u8>>>;

    /// Returns the number of elements in the map.
    fn len(&self) -> usize;

    /// Returns true if the map contains no elements, false otherwise.
    fn is_empty(&self) -> bool;
}