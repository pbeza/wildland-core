//
// Wildland Project
//
// Copyright © 2022 Golem Foundation
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3 as published by
// the Free Software Foundation.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use super::result::LssResult;

pub trait LocalSecureStorage {
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

    /// Removes a key from the map, returning the value at the key if the key was previously in the map.
    fn remove(&self, key: String) -> LssResult<Option<Vec<u8>>>;

    /// Returns the number of elements in the map.
    fn len(&self) -> LssResult<usize>;

    /// Returns true if the map contains no elements, false otherwise.
    fn is_empty(&self) -> LssResult<bool>;
}
