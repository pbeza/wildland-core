pub type FingerPrint = Vec<u8>;
pub type ForestPath = String;

/// Common interface representing a single `Forest` in the system
pub trait Forest {
    fn get_name() -> String;
    fn get_owner() -> FingerPrint;
    fn get_devices() -> String;
}

pub struct ForestImpl {}
