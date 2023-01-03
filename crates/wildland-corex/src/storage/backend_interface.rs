pub trait StorageBackend {
    fn readdir(&self);
}
