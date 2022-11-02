use wildland_corex::storage::StorageTemplate as InnerStorageTemplate;

#[derive(Debug, Clone)]
pub struct StorageTemplate {
    inner: InnerStorageTemplate,
}

impl StorageTemplate {
    pub(crate) fn new(inner: InnerStorageTemplate) -> Self {
        Self { inner }
    }

    pub(crate) fn inner(&self) -> &InnerStorageTemplate {
        &self.inner
    }

    pub fn stringify(&self) -> String {
        format!("Storage Template (uuid: {})", self.inner.uuid())
    }
}
