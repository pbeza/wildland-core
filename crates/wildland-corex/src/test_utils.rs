use mockall::mock;
use uuid::Uuid;

use crate::catlib_service::entities::{
    Bridge,
    ContainerManifest,
    ContainerPath,
    ForestManifest,
    Identity,
    Signers,
};
use crate::catlib_service::error::CatlibResult;

mock! {
    #[derive(Debug)]
    pub Forest {}
    impl Clone for Forest {
        fn clone(&self) -> Self;
    }
    impl ForestManifest for Forest {
        fn add_signer(&mut self, signer: Identity) -> CatlibResult<bool>;
        fn del_signer(&mut self, signer: Identity) -> CatlibResult<bool>;
        fn containers(&self) -> CatlibResult<Vec<Box<dyn ContainerManifest>>>;
        fn update(&mut self, data: Vec<u8>) -> CatlibResult<()>;
        fn delete(&mut self) -> CatlibResult<bool>;
        fn create_container(&self, name: String) -> CatlibResult<Box<dyn ContainerManifest>>;
        fn create_bridge( &self,
            path: ContainerPath,
            link_data: Vec<u8>,
        ) -> CatlibResult<Box<dyn Bridge>>;
        fn find_bridge(&self, path: ContainerPath) -> CatlibResult<Box<dyn Bridge>>;
        fn find_containers( &self,
            paths: Vec<ContainerPath>,
            include_subdirs: bool,
        ) -> CatlibResult<Vec<Box<dyn ContainerManifest>>>;
        fn data(&mut self) -> CatlibResult<Vec<u8>>;
        fn uuid(&self) -> Uuid;
        fn owner(&self) -> Identity;
        fn signers(&mut self) -> CatlibResult<Signers>;
    }
}
