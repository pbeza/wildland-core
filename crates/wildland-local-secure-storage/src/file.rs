use crate::api::LocalSecureStorage;
use crate::LSSResult;
use rustbreak::deser::Yaml;
use rustbreak::PathDatabase;
use std::path::PathBuf;

type FileLSSData = std::collections::HashMap<String, Vec<u8>>;

#[derive(Debug)]
pub struct FileLSS {
    db: Box<PathDatabase<FileLSSData, Yaml>>,
}

impl FileLSS {
    pub fn new(path: PathBuf) -> LSSResult<Self> {
        let db = PathDatabase::load_from_path_or_default(path)?;
        Ok(Self { db: Box::new(db) })
    }
}

impl LocalSecureStorage for FileLSS {
    fn insert(&self, key: String, value: Vec<u8>) -> LSSResult<Option<Vec<u8>>> {
        let prev_value = self.db.read(|db| db.get(&key).map(|v| v.to_vec()))?;
        self.db.write(|db| db.insert(key, value))?;
        Ok(prev_value)
    }

    fn get(&self, key: String) -> LSSResult<Option<Vec<u8>>> {
        let result = self.db.read(|db| db.get(&key).map(|v| v.to_vec()))?;
        Ok(result)
    }

    fn contains_key(&self, key: String) -> LSSResult<bool> {
        let result = self.db.read(|db| db.contains_key(&key))?;
        Ok(result)
    }

    fn keys(&self) -> LSSResult<Vec<String>> {
        let result: Vec<String> = self
            .db
            .read(|db| db.keys().map(|k| k.to_string()).collect())?;
        Ok(result)
    }

    fn remove(&self, key: String) -> LSSResult<Option<Vec<u8>>> {
        let prev_value = self.db.read(|db| db.get(&key).map(|v| v.to_vec()))?;
        self.db.write(|db| db.remove(&key))?;
        Ok(prev_value)
    }

    fn len(&self) -> LSSResult<usize> {
        let result = self.db.read(|db| db.len())?;
        Ok(result)
    }

    fn is_empty(&self) -> LSSResult<bool> {
        let result = self.db.read(|db| db.is_empty())?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::api::LocalSecureStorage;
    use crate::file::FileLSS;
    use tempfile::tempdir;

    fn create_file_lss() -> FileLSS {
        let dir = tempdir().expect("Could not create temporary dir");
        let file_path = dir.path().join("lss-test.yaml");
        FileLSS::new(file_path).unwrap()
    }

    #[test]
    fn should_insert_new_value_and_return_none() {
        let lss = create_file_lss();
        let result = lss.insert("foo".to_string(), b"bar".to_vec()).unwrap();

        assert!(result.is_none())
    }

    #[test]
    fn should_update_value_and_return_previous() {
        let lss = create_file_lss();
        lss.insert("foo".to_string(), b"bar".to_vec()).unwrap();
        let result = lss.insert("foo".to_string(), b"baz".to_vec()).unwrap();

        assert_eq!(result.unwrap(), b"bar".to_vec())
    }

    #[test]
    fn should_get_inserted_value() {
        let lss = create_file_lss();
        lss.insert("foo".to_string(), b"bar".to_vec()).unwrap();
        let result = lss.get("foo".to_string()).unwrap();

        assert_eq!(result.unwrap(), b"bar".to_vec())
    }

    #[test]
    fn should_get_none_when_no_value_presented() {
        let lss = create_file_lss();
        let result = lss.get("foo".to_string()).unwrap();

        assert!(result.is_none())
    }

    #[test]
    fn should_return_true_when_contains_key() {
        let lss = create_file_lss();
        lss.insert("foo".to_string(), b"bar".to_vec()).unwrap();
        let result = lss.contains_key("foo".to_string()).unwrap();

        assert!(result)
    }

    #[test]
    fn should_return_false_when_does_not_contain_key() {
        let lss = create_file_lss();
        let result = lss.contains_key("foo".to_string()).unwrap();

        assert!(!result)
    }

    #[test]
    fn should_return_list_of_keys() {
        let lss = create_file_lss();
        lss.insert("foo".to_string(), b"bar".to_vec()).unwrap();
        lss.insert("baz".to_string(), b"bar".to_vec()).unwrap();
        let result = lss.keys().unwrap();

        assert_eq!(result.len(), 2);
        assert!(result
            .iter()
            .all(|item| item.as_str() == "foo" || item.as_str() == "baz"))
    }

    #[test]
    fn should_return_empty_list_of_keys_when_db_is_empty() {
        let lss = create_file_lss();
        let result = lss.keys().unwrap();

        assert!(result.is_empty())
    }

    #[test]
    fn should_remove_return_none_when_key_not_presented() {
        let lss = create_file_lss();
        let result = lss.remove("foo".to_string()).unwrap();

        assert!(result.is_none())
    }

    #[test]
    fn should_remove_and_return_previous_value() {
        let lss = create_file_lss();
        lss.insert("foo".to_string(), b"bar".to_vec()).unwrap();
        let result = lss.remove("foo".to_string()).unwrap();
        let contains_foo = lss.contains_key("foo".to_string()).unwrap();

        assert_eq!(result.unwrap(), b"bar".to_vec());
        assert!(!contains_foo)
    }

    #[test]
    fn should_return_keys_len() {
        let lss = create_file_lss();
        lss.insert("foo".to_string(), b"bar".to_vec()).unwrap();
        lss.insert("baz".to_string(), b"bar".to_vec()).unwrap();
        let result = lss.len().unwrap();

        assert_eq!(result, 2)
    }

    #[test]
    fn should_return_zero_len_when_db_is_empty() {
        let lss = create_file_lss();
        let result = lss.len().unwrap();

        assert_eq!(result, 0)
    }

    #[test]
    fn should_return_false_if_db_is_not_empty() {
        let lss = create_file_lss();
        lss.insert("foo".to_string(), b"bar".to_vec()).unwrap();
        lss.insert("baz".to_string(), b"bar".to_vec()).unwrap();
        let result = lss.is_empty().unwrap();

        assert!(!result)
    }

    #[test]
    fn should_return_true_when_db_is_empty() {
        let lss = create_file_lss();
        let result = lss.is_empty().unwrap();

        assert!(result)
    }
}
