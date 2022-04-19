use uuid::Uuid;

pub trait BridgeACL {}

pub trait BridgeAPI {
    fn create_bridge(forest_id: Uuid, path: &str, link_data: &[u8]) -> Uuid;
    fn set_bridge_acl(bridge_id: Uuid, pubkeys: &[&str]) -> Result<(), i32>;
    fn get_bridge_acl(bridge_id: Uuid) -> dyn BridgeACL;
    fn del_bridge_acl(bridge_id: Uuid, pubkeys: &[&str]) -> Result<(), i32>;
    fn remove_bridge(bridge_id: Uuid) -> Result<(), i32>;
    fn remove_bridge_by_path(path: &str) -> Result<(), i32>;
}
