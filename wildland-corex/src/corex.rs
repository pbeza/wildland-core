use std::collections::HashMap;

use crate::bridge::{Bridge, BridgeCredentials, BridgePath};
use crate::catalog::{Catalog, CatalogCredentials};
use crate::error::CoreXError;
use crate::WalletType;
use wildland_catlib::{ForestImpl, ForestPath};

/// TODO:
pub struct CoreX {
    _forests: HashMap<ForestPath, ForestImpl>,
    _bridges: HashMap<BridgePath, Bridge>,
}

impl CoreX {
    /// Creates local bridge
    pub fn create_local_bridge(_creds: BridgeCredentials, _path: String) {
        todo!()
    }

    /// For cargo path whill be hardcoded : wildland:/cargo:/users/
    /// bridge :cargo will be stored in etherium and will be
    /// pointing to a catalog hosted and maintained by wildland which will store
    /// [Forest discovery and addressing](https://hackmd.io/PojtmsxQTiuB03sfOfvqbQ)
    pub fn create_forest(
        &self,
        _wallet_type: WalletType,
        _forest_name: String,
        _forest_path: ForestPath,
        _catalog: Catalog,
        _catalog_credentials: CatalogCredentials,
    ) -> Result<ForestImpl, CoreXError> {
        todo!()
    }

    /// TODO: is it required?
    pub fn store_secret(_key: String) {
        todo!()
    }
}
