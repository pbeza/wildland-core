use uuid::Uuid;

pub enum Permissions {
    Meta,
    Paths,
    Data,
}

pub trait ContainerACL {}

pub trait Container {
    fn create_container(&self, forest_id: Uuid, data_encryption_key: &[u8]) -> Result<Uuid, i32>;
    fn add_container_path(&self, container_id: Uuid, path: &str) -> Result<Uuid, i32>;
    /// Updating a Path will require a Del+Add transaction
    fn del_container_path(&self, path_id: Uuid) -> Result<(), i32>;
    fn del_container_path_by_string(&self, path: &str) -> Result<(), i32>;
    fn set_container_acl(
        &self,
        container_id: Uuid,
        pubkeys: &[&str],
        permissions: &[Permissions],
    ) -> Result<(), Uuid>;
    fn get_container_acl(container_id: Uuid) -> dyn ContainerACL;
    fn del_container_acl(acl_id: Uuid) -> Result<(), i32>;
    fn del_container_aclby_key(container_id: Uuid, pubkeys: &[&str]) -> Result<(), i32>;
    fn get_container(container_id: Uuid) -> Self;
    // fn GetContainersByPathId(path_id: &[Uuid]) -> Vec<Self>;
    // fn GetContainersByPath(
    //     Uuid forest_id
    //     []String path,
    //     Bool [recursively|include_subdirs],
    // ) -> []Container;
    // fn GetContainersByTemplate(
    //     Uuid template_id,
    //     Uuid forest_id
    // ) -> []Container;
    fn del_container(container_id: Uuid) -> Result<(), i32>;
    fn del_containers_by_path(path: &str, recursively: bool) -> Result<(), i32>;
}
