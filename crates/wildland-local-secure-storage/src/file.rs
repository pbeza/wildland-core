use std::path::PathBuf;
use rustbreak::deser::Yaml;
use rustbreak::PathDatabase;
use crate::api::LocalSecureStorage;
use crate::LSSResult;

type FileLSSData = std::collections::HashMap<String, Vec<u8>>;

struct FileLSS {
    db: PathDatabase<FileLSSData, Yaml>
}

impl FileLSS {
    fn new(path: PathBuf) -> Self {
        Self {
            db: PathDatabase::load_from_path_or_default(path)
                .expect("Could not create FileLSS from path")
        }
    }
}

impl LocalSecureStorage for FileLSS {
    fn insert(&mut self, key: String, value: Vec<u8>) -> LSSResult<Option<Vec<u8>>> {
        let prev_value = self.db.read(|db| db.get(&key)
            .map(|v| v.to_vec()))?;
        self.db.write(|db| db.insert(key, value))?;
        Ok(prev_value)
    }

    fn get(&self, key: String) -> LSSResult<Option<Vec<u8>>> {
        let result = self.db.read(|db| db.get(&key)
            .map(|v| v.to_vec()))?;
        Ok(result)
    }

    fn contains_key(&self, key: String) -> LSSResult<bool> {
        let result = self.db.read(|db| db.contains_key(&key))?;
        Ok(result)
    }

    fn keys(&self) -> LSSResult<Vec<String>> {
        let mut result: Vec<String> = self.db.read(|db| db.keys()
            .map(|k| k.to_string())
            .collect())?;
        result.sort();
        Ok(result)
    }

    fn remove(&mut self, key: String) -> LSSResult<Option<Vec<u8>>> {
        todo!()
    }

    fn len(&self) -> LSSResult<usize> {
        todo!()
    }

    fn is_empty(&self) -> LSSResult<bool> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;
    use crate::api::LocalSecureStorage;
    use crate::file::FileLSS;

    fn create_file_lss() -> FileLSS {
        let dir = tempdir().expect("Could not create temporary dir");
        let file_path = dir.path().join("lss-test.yaml");
        FileLSS::new(file_path)
    }

    #[test]
    fn should_insert_new_value_and_return_none() {
        let mut lss = create_file_lss();
        let result = lss.insert("foo".to_string(), b"bar".to_vec()).unwrap();

        assert!(result.is_none())
    }

    #[test]
    fn should_update_value_and_return_previous() {
        let mut lss = create_file_lss();
        lss.insert("foo".to_string(), b"bar".to_vec()).unwrap();
        let result = lss.insert("foo".to_string(), b"baz".to_vec()).unwrap();

        assert_eq!(result.unwrap(), b"bar".to_vec())
    }

    #[test]
    fn should_get_inserted_value() {
        let mut lss = create_file_lss();
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
        let mut lss = create_file_lss();
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
    fn should_return_sorted_list_of_keys() {
        let mut lss = create_file_lss();
        lss.insert("foo".to_string(), b"bar".to_vec()).unwrap();
        lss.insert("baz".to_string(), b"bar".to_vec()).unwrap();
        let result = lss.keys().unwrap();

        assert!(result.iter().eq(vec!["baz".to_string(), "foo".to_string()].iter()));
    }

    #[test]
    fn should_return_empty_list_of_keys_when_db_is_empty() {
        let lss = create_file_lss();
        let result = lss.keys().unwrap();

        assert!(result.is_empty())
    }
}