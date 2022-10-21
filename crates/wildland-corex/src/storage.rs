use std::rc::Rc;
use uuid::Uuid;

pub trait StorageTemplateTrait {
    fn uuid(&self) -> Uuid;
    fn data(&self) -> Vec<u8>;
}

#[derive(Clone)]
pub struct StorageTemplate {
    inner: Rc<dyn StorageTemplateTrait>,
}

impl StorageTemplate {
    pub fn uuid(&self) -> Uuid {
        self.inner.uuid()
    }

    pub fn data(&self) -> Vec<u8> {
        self.inner.data()
    }

    pub fn with_template(storage_template: Rc<dyn StorageTemplateTrait>) -> Self {
        Self {
            inner: storage_template,
        }
    }
}
