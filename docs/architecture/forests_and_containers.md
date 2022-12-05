# Forests and Containers API
This document describes Forests and Containers concepts in terms of software design and API.


## Design
Model diagrams and flow charts related to Containers and Forests from the Wildland-Core perspective.


### Forest and container model in Wildland Core
The following model should be treated as a logical representation of the Container and Forest in the Wildland Core. It is not an exact representation of the data model used in any database. This model can be optimised per the database type to achieve the best performance and security.

```mermaid
classDiagram
ForestEntity --> ContainerEntity
ContainerManager --> ContainerEntity
ContainerEntity : Uuid uuid
ContainerEntity : Uuid forest_uuid
ContainerEntity : String name
ContainerEntity : Vec<String> paths

ForestEntity : Uuid uuid
ForestEntity : Identity owner
ForestEntity : Vec< Signer > signers
ForestEntity : Vec< u8 > data
ForestEntity : HashMap< Uuid, ContainerEntity > containers
ForestEntity : HashMap< Uuid, BridgeEntity > bridges

ContainerManager : HashSet< Uuid > mounted_containers
```


### Example Rust structure for Forest data model in CatLib
```rust
pub struct ForestEntity {
    /// Unique at CatLib application level.
    /// 
    uuid: Uuid,

    /// There's only one owner (one public key per forest).
    /// 
    owner: Identity,

    /// For multi-device purpose (and optionally for the forest sharing mechanism).
    /// Each device has to sign the manifest.
    /// 
    signers: Vec<Signer>,

    /// Arbitrary CatLib-indepenend data. For instance Storage backends (serialized to json) etc.
    /// 
    data: Vec<u8>,

    /// Collection of containers that builds the forest.
    /// 
    containers: HashMap<Uuid, ContainerEntity>,

    /// Collection of bridges that links the forest to other forests.
    /// Bridge works as a link between forests - instead of StorageBackend
    /// contains some sort of a link to another container.
    /// 
    bridges: HashMap<Uuid, BridgeEntity>,
}

/// `ForestEntity` from CatLib should conform to trait `ForestManifest` from CoreX.
/// 
impl ForestManifest for ForestEntity { 
    // ...
}
```


### Example Rust structure for Container data model in CatLib
```rust
pub struct ContainerEntity { 
    /// Container unique UUID.
    /// 
    uuid: Uuid,

    /// Uuid (Each container has exactly one forest and one forest has many containers).
    /// 
    forest_uuid: Uuid,
    
    /// User defined container name.
    /// 
    name: String,

    /// Container claims paths within the given forest. Given directory in a
    /// file tree consists of files from different mounted containers.
    /// 
    paths: Vec<ContainerPath>
}

/// `ContainerEntity` from CatLib should conform to trait `ContainerManifest` from CoreX.
/// 
impl ContainerManifest for ContainerEntity { 
    // ...
}
```


### Example Rust structure for Container Management data model in CoreX
```rust
/// This structure preserve local state of the containers (application-wise).
/// 
pub struct ContainersState {
    /// Collection of containers locally mounted in the File Tree.
    /// 
    mounted_containers: HashSet<Uuid>,
}

impl ContainersManager for ContainersState {
    // ...
}
```


### Crates dependencies
```mermaid
classDiagram
CargoLib --> CatLib: Creates Cat-Lib instances.
CargoLib --> CoreX: Provides CoreX with Cat-Lib instances.
CatLib --> CoreX: Implements tratis of Containers and Forests.
CargoLib : Entrypoint for the CargoLib application.
CoreX : Forest and container API.
CatLib : Data model.
```


## CoreX Forest API.
It is recommended that `ForestManifest` trait implementation should synchronize with the database on every method call in order to keep the state up-to-date.

```rust
/// `ForestManifest` trait is an API providing methods needed to operate on the forest's
/// state. It should be implemented by Cat-Lib instance and should be
/// treated as a kind of a proxy layer between Wildland Core and the external
/// persistent data storage instance (for e.g. database).
/// 
pub trait ForestManifest {
    /// Add manifest Signer
    ///
    /// Returns whether the value was newly inserted. That is:
    ///
    /// - If the signer did not previously exist, `true` is returned.
    /// - If the signer already exists, `false` is returned.
    /// 
    fn add_signer(&mut self, signer: Identity) -> Result<bool, CatlibError>;

    /// Delete manifest Signer
    ///
    /// Returns whether the value was already present. That is:
    ///
    /// - If the signer did not previously exist, `false` is returned.
    /// - If the signer existed in the set, `true` is returned.
    /// 
    fn del_signer(&mut self, signer: Identity) -> Result<bool, CatlibError>;

    /// Return list of Forest Containers
    fn containers(&self) -> Result<Vec<Box<dyn Container>>, CatlibError>;

    /// Set Forest arbitrary data
    /// 
    fn update(&mut self, data: Vec<u8>) -> Result<&mut dyn Forest, CatlibError>;

    /// Delete Forest
    ///
    /// **WARN: The underlying objects are not removed recursively**
    /// 
    fn delete(&mut self) -> Result<bool, CatlibError>;

    /// Create an empty container, bound to the Forest.
    ///
    /// To set container paths, use [`Container::add_path`]
    /// 
    fn create_container(&self, name: String) -> Result<Box<dyn Container>, CatlibError>;

    /// Create a Bridge object with arbitrary link data to another Forest.
    ///
    /// The aforementioned link data will be defined by the D/FS module.
    /// 
    fn create_bridge(
        &self,
        path: ContainerPath,
        link_data: Vec<u8>,
    ) -> Result<Box<dyn Bridge>, CatlibError>;

    /// Return bridge that matches the given [`ContainerPath`].
    fn find_bridge(&self, path: ContainerPath) -> Result<Box<dyn Bridge>, CatlibError>;

    /// Retrieve Containers that match given [`ContainerPath`]s.
    ///
    /// If `include_subdirs` is `true`, then the [`ContainerPath`]s are treated as Path prefixes
    /// and not absolute paths.
    /// 
    fn find_containers(
        &self,
        paths: Vec<ContainerPath>,
        include_subdirs: bool,
    ) -> Result<Vec<Box<dyn Container>>, CatlibError>;
}
```


## CoreX Container API for Cat-Lib implementation.
It is recommended that `ContainerManifest` trait implementation should synchronize with the database on every method call in order to keep the state up-to-date.

```rust
/// `ContainerManifest` trait is an API providing methods needed to manipulate container's
/// configuration state. It should be implemented by Cat-Lib instance and should be
/// treated as a kind of a proxy layer between Wildland Core and the external
/// persistent data storage instance (for e.g. database).
/// 
pub trait ContainerManifest {
    /// Lists the storages that the given container use in order to keep the data.
    /// 
    fn get_storages(&self) -> Result<Vec<&dyn StorageBackend>, GetStoragesError>;

    /// Removes the given storage backend from the container.
    /// This operation should involve a secure way of erasing data from the storage.
    /// 
    fn delete_storage(&mut self, storage: &dyn StorageBackend) -> Result<(), DeleteStorageError>;

    /// This operation adds a given Storage Backend to the container.
    /// The procedure involves starting the data sync mechanism between the new storage
    /// and the other present storages.
    /// 
    /// Container can have multiple storages. Given container should has exact copies
    /// of the data on every storage.
    /// 
    fn add_storage(&mut self, storage: &dyn StorageBackend) -> Result<(), AddStorageError>;

    /// Checks whether the given container handle is deleted.
    /// 
    fn is_deleted(&self) -> bool;

    /// Returns a printable description of the given container.
    /// 
    fn stringify(&self) -> String;

    /// Deletes all paths that the given container contains.
    /// In result the container is considered deleted afterwards.
    /// Container should be treated as a "shared pointer" - once the
    /// last path is deleted the container should be moved to 
    /// some sort of a "trash bin".
    /// 
    fn delete(&mut self, catlib_service: &CatLibService) -> Result<(), CatlibError>;

    /// Returns the unique ID of the container.
    /// 
    fn uuid(&self) -> Uuid;

    /// Returns true if path was actually added, false otherwise.
    /// 
    fn add_path(&mut self, path: String) -> Result<bool, CatlibError>;

    /// Removes the given path. Returns true if the path was actually deleted,
    /// false if the path was not present within the container.
    /// 
    fn delete_path(&mut self, path: String) -> Result<bool, CatlibError>;

    /// Lists all the paths from the given container.
    /// 
    fn get_paths(&self) -> Result<Vec<String>, CatlibError>;

    /// User provided name of the container.
    /// 
    fn get_name(&self) -> String;

    /// Sets the user provided name for the container.
    /// This operation involves updating at least the local storage.
    /// 
    fn set_name(&mut self, new_name: String) -> Result<(), ContainerError>;
}
```


## CoreX Container API for mount procedure.
```rust
/// This trait should be implemented in CoreX or other special crate that is responsible
/// for mounting the container within the File Tree. Container Manager is an API for the
/// object representing the local (device) state of the forest and containers. It keeps
/// the information about containers mounted locally.
/// 
pub trait ContainersManager {
    /// Mount creates locally the representation of the container (files/directories structures).
    /// The procedure updates only the file tree, it doesn't sync the container data itself.
    ///
    /// This procedure is considered to work in background.
    /// 
    fn mount(&mut self, container: &dyn ContainerManifest) -> Result<(), ContainerMountError>;

    /// The operation oposit to mount. It removes the container from the file tree representation.
    /// 
    /// This procedure is considered to work in background.
    /// 
    fn unmount(&mut self, container: &dyn ContainerManifest) -> Result<(), ContainerUnmountError>;

    /// Checks whether the given container was mounted in the file tree.
    /// 
    fn is_mounted(&self, container: &dyn ContainerManifest) -> bool;
}
```


## CargoLib API
`ContainerManager`, `ForestManifest` and `ContainerManifest` are provided by `CargoUser` object. This API requires rusty-bind to provide users with `Box` type support.

```rust
mod ffi {
    //
    // ContainerManager
    //
    fn mount(self: &mut ContainersManager, container: &dyn ContainerManifest) -> Result<(), ContainerMountError>;
    fn unmount(self: &mut ContainersManager, container: &dyn ContainerManifest) -> Result<(), ContainerUnmountError>;
    fn is_mounted(self: &mut ContainersManager, container: &dyn ContainerManifest) -> bool;

    //
    // ForestManifest
    //
    fn get_storages(self: &Arc<Mutex<ForestManifest>>) -> Result<Vec<&dyn StorageBackend>, GetStoragesError>;
    fn delete_storage(self: &Arc<Mutex<ForestManifest>>, storage: &dyn StorageBackend) -> Result<(), DeleteStorageError>;
    fn add_storage(self: &Arc<Mutex<ForestManifest>>, storage: &dyn StorageBackend) -> Result<(), AddStorageError>;
    fn is_deleted(self: &Arc<Mutex<ForestManifest>>) -> bool;
    fn stringify(self: &Arc<Mutex<ForestManifest>>) -> String;
    fn delete(self: &Arc<Mutex<ForestManifest>>, catlib_service: &CatLibService) -> Result<(), CatlibError>;
    fn uuid(self: &Arc<Mutex<ForestManifest>>) -> Uuid;
    fn add_path(self: &Arc<Mutex<ForestManifest>>, path: String) -> Result<bool, CatlibError>;
    fn delete_path(self: &Arc<Mutex<ForestManifest>>, path: String) -> Result<bool, CatlibError>;
    fn get_paths(self: &Arc<Mutex<ForestManifest>>) -> Result<Vec<String>, CatlibError>;
    fn get_name(self: &Arc<Mutex<ForestManifest>>) -> String;
    fn set_name(self: &Arc<Mutex<ForestManifest>>, new_name: String) -> Result<(), ContainerError>;

    //
    // ContainerManifest
    //
    fn add_signer(self: &Arc<Mutex<ContainerManifest>>, signer: Identity) -> Result<bool, CatlibError>;
    fn del_signer(self: &Arc<Mutex<ContainerManifest>>, signer: Identity) -> Result<bool, CatlibError>;
    fn containers(self: &Arc<Mutex<ContainerManifest>>) -> Result<Vec<Box<dyn Container>>, CatlibError>;
    fn update(self: &Arc<Mutex<ContainerManifest>>, data: Vec<u8>) -> Result<&mut dyn Forest, CatlibError>;
    fn delete(self: &Arc<Mutex<ContainerManifest>>) -> Result<bool, CatlibError>;
    fn create_container(self: &Arc<Mutex<ContainerManifest>>, name: String) -> Result<Box<dyn Container>, CatlibError>;
    fn find_bridge(self: &Arc<Mutex<ContainerManifest>>, path: ContainerPath) -> Result<Box<dyn Bridge>, CatlibError>;
    fn find_containers(
        self: &Arc<Mutex<ContainerManifest>>,
        paths: Vec<ContainerPath>,
        include_subdirs: bool,
    ) -> Result<Vec<Box<dyn Container>>, CatlibError>;

}
```
