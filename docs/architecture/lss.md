# Local Secure Storage

## Description

LSS is a platform specific component that is only exposed from Wildland as a
set of traits. That means each platform (os) have to have their own
implementations that are then passed as an instance to wildland.

LSS data is persistant for specific device.

LSS is responsible for storing security oriented data, for example:

- private keys (device, forest, master identity)
- storage definitions

Example implementations of LSS:

- Keychain CRUD wrapper (ios/osx/android)
- Keyring CRUD wrapper (linux)
- Credential Manager CRUD wrapper (windows)

## Trait API

This is the api that has to be provided by the LSS object in order to be usable
by wildland. Note that LSS is not exposed outside as a public API.

```rust
    /// Inserts a key-value pair into the LSS.
    /// If the map did not have this key present, None is returned.
    /// If the map did have this key present, the value is updated, and the old value is returned.
    fn insert(&self, key: String, value: Vec<u8>) -> LssResult<Option<Vec<u8>>>;

    /// Returns a copy of the value corresponding to the key.
    fn get(&self, key: String) -> LssResult<Option<Vec<u8>>>;

    /// Returns true if the map contains a value for the specified key.
    fn contains_key(&self, key: String) -> LssResult<bool>;

    /// Returns all keys in arbitrary order.
    fn keys(&self) -> LssResult<Vec<String>>;

    /// Returns all keys starting with the given prefix.
    fn keys_starting_with(&self, prefix: String) -> LssResult<Vec<String>>;

    /// Removes a key from the map, returning the value at the key if the key was previously in the map.
    fn remove(&self, key: String) -> LssResult<Option<Vec<u8>>>;

    /// Returns the number of elements in the map.
    fn len(&self) -> LssResult<usize>;

    /// Returns true if the map contains no elements, false otherwise.
    fn is_empty(&self) -> LssResult<bool>;
```
