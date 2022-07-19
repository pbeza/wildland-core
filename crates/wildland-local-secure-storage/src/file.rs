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
        let prev_value = self.db.read(|db| db.get(&key).map(|v| v.to_vec()))?;
        self.db.write(|db| db.insert(key, value))?;
        Ok(prev_value)
    }

    fn get(&self, key: String) -> Option<Vec<u8>> {
        todo!()
    }

    fn contains_key(&self, key: String) -> bool {
        todo!()
    }

    fn keys(&self) -> Vec<String> {
        todo!()
    }

    fn remove(&mut self, key: String) -> LSSResult<Option<Vec<u8>>> {
        todo!()
    }

    fn len(&self) -> usize {
        todo!()
    }

    fn is_empty(&self) -> bool {
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
    fn should_insert_new_value() {
        let mut lss = create_file_lss();
        let result = lss.insert("foo".to_string(), b"bar".to_vec()).unwrap();

        assert!(result.is_none())
    }
}