use uuid::Uuid;

type StorageId = Uuid;
type ContainerId = Uuid;
type TemplateId = Uuid;

pub enum Permissions {
    RO,
    RW,
}

pub trait Storage {}

pub trait StorageACL {}

pub trait StorageAPI {
    fn add_container_storage(
        &self,
        container_id: ContainerId,
        template_id: TemplateId,
        storage_data: &str,
    ) -> StorageId;
    fn set_storage_acl(
        &self,
        storage_id: StorageId,
        pubkeys: &[&str],
        permissions: &[Permissions],
    ) -> Result<(), i32>;
    fn get_storage_acl(&self, storage_id: StorageId) -> dyn StorageACL;
    fn del_storage_acl(&self, storage_id: StorageId, pubkeys: &[&str]) -> Result<(), i32>;
    // fn get_storages_by_container(&self, container_id: ContainerId) -> Vec<dyn Storage>;
    // fn get_storages_by_template(&self, template_id: TemplateId) -> Vec<dyn Storage>;
    // fn get_storages_by_acl(&self, pubkeys: &[&str]) -> Vec<dyn Storage>;
    fn remove_storage(&self, storage_id: StorageId) -> Result<(), i32>;
    fn remove_storages_by_template(&self, template_id: TemplateId) -> Result<(), i32>;
    fn remove_storages_from_container(&self, container_id: ContainerId) -> Result<(), i32>;
}
