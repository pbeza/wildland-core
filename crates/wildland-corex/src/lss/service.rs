use std::fmt::{Debug, Display};

use super::{api::LocalSecureStorage, result::LssResult};
use crate::{ForestRetrievalError, LssError, WildlandIdentity, DEFAULT_FOREST_KEY};
use serde::{de::DeserializeOwned, Serialize};
use uuid::Uuid;
use wildland_catlib::{Forest, IForest, Identity};
use wildland_crypto::identity::SigningKeypair;

#[derive(Clone)]
pub struct LssService {
    lss: &'static dyn LocalSecureStorage,
}

const THIS_DEVICE_KEYPAIR_KEY: &str = "wildland.device.keypair";
const THIS_DEVICE_NAME_KEY: &str = "wildland.device.name";

impl LssService {
    pub fn new(lss: &'static dyn LocalSecureStorage) -> Self {
        tracing::debug!("created new instance");
        Self { lss }
    }

    #[tracing::instrument(level = "debug", skip(self, wildland_identity))]
    pub fn save_identity(&self, wildland_identity: &WildlandIdentity) -> LssResult<bool> {
        let key = match wildland_identity {
            WildlandIdentity::Forest(_, _) => wildland_identity.to_string(),
            WildlandIdentity::Device(device_name, _) => {
                self.serialize_and_save(THIS_DEVICE_NAME_KEY, &device_name)?;

                THIS_DEVICE_KEYPAIR_KEY.to_owned()
            }
        };
        self.serialize_and_save(key, &wildland_identity.get_keypair())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn get_this_device_name(&self) -> LssResult<Option<String>> {
        self.get_parsed(THIS_DEVICE_NAME_KEY)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn get_this_device_keypair(&self) -> LssResult<Option<SigningKeypair>> {
        self.get_parsed(THIS_DEVICE_KEYPAIR_KEY)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn get_default_forest_keypair(&self) -> LssResult<Option<SigningKeypair>> {
        self.get_parsed(DEFAULT_FOREST_KEY)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_this_device_identity(&self) -> LssResult<Option<WildlandIdentity>> {
        tracing::trace!("Getting this device identity.");

        let optional_this_device_identity = self.get_this_device_keypair()?;
        optional_this_device_identity.map_or(Ok(None), |this_device_identity| {
            let device_name = self
                .get_this_device_name()?
                .ok_or_else(|| LssError("Could not retrieve device name from LSS".to_owned()))?;
            Ok(Some(WildlandIdentity::Device(
                device_name,
                this_device_identity,
            )))
        })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_default_forest(&self) -> Result<Option<WildlandIdentity>, ForestRetrievalError> {
        tracing::trace!("Getting default forest identity.");

        let optional_default_forest_identity = self.get_default_forest_keypair()?;
        optional_default_forest_identity.map_or(Ok(None), |default_forest_value| {
            Ok(Some(WildlandIdentity::Forest(0, default_forest_value)))
        })
    }

    #[tracing::instrument(level = "debug", skip(self, forest))]
    pub fn save_forest_uuid(&self, forest: &Forest) -> LssResult<bool> {
        tracing::trace!("Saving forest uuid");
        self.serialize_and_save(forest.owner().encode(), &forest.uuid())
    }

    #[tracing::instrument(level = "debug", skip(self, forest_identity))]
    pub fn get_forest_uuid_by_identity(
        &self,
        forest_identity: &WildlandIdentity,
    ) -> LssResult<Option<Uuid>> {
        self.get_parsed(Identity::from(forest_identity.get_public_key()).encode())
    }

    #[tracing::instrument(level = "debug", skip(self, obj))]
    fn serialize_and_save(
        &self,
        key: impl Display + Debug,
        obj: &impl Serialize,
    ) -> LssResult<bool> {
        self.lss
            .insert(
                key.to_string(),
                serde_json::to_vec(obj)
                    .map_err(|e| LssError(format!("Could not serialize object: {e}")))?,
            )
            .map(|bytes| bytes.is_some())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn get_parsed<'a, T: DeserializeOwned>(
        &self,
        key: impl Display + Debug,
    ) -> LssResult<Option<T>> {
        self.lss.get(key.to_string()).and_then(|opt_bytes| {
            opt_bytes.map_or(Ok(None), |bytes| {
                serde_json::from_slice(bytes.as_slice())
                    .map_err(|e| LssError(format!("Could not parse LSS entry: {e}")))
            })
        })
    }
}
