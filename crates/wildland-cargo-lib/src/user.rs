//
// Wildland Project
//
// Copyright © 2022 Golem Foundation
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3 as published by
// the Free Software Foundation.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use uuid::Uuid;
use wildland_corex::catlib_service::error::CatlibError;
use wildland_corex::catlib_service::{CatLibService, DeviceMetadata, ForestMetaData};
use wildland_corex::{
    ContainerManager,
    CryptoError,
    Identity,
    LssService,
    MasterIdentity,
    MnemonicPhrase,
};

use crate::api::cargo_user::CargoUser;
use crate::api::config::FoundationStorageApiConfig;
use crate::errors::{UserCreationError, UserRetrievalError};

pub fn generate_random_mnemonic() -> Result<MnemonicPhrase, CryptoError> {
    wildland_corex::generate_random_mnemonic()
}

pub enum CreateUserInput {
    Mnemonic(Box<MnemonicPhrase>),
    Entropy(Vec<u8>),
}

/// This struct contains User functionalities but in contrast to [`super::api::UserApi`] it is not exposed through FFI
///
#[derive(Clone)]
pub(crate) struct UserService {
    lss_service: LssService,
    catlib_service: CatLibService,
    fsa_config: FoundationStorageApiConfig,
    container_manager: ContainerManager,
}

impl UserService {
    pub(crate) fn new(
        lss_service: LssService,
        catlib_service: CatLibService,
        fsa_config: FoundationStorageApiConfig,
        container_manager: ContainerManager,
    ) -> Self {
        Self {
            lss_service,
            catlib_service,
            fsa_config,
            container_manager,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) fn create_user(
        &self,
        input: CreateUserInput,
        device_name: String,
    ) -> Result<CargoUser, UserCreationError> {
        tracing::trace!("Checking whether user exists.");
        match self.get_user() {
            Ok(_) => return Err(UserCreationError::UserAlreadyExists),
            Err(UserRetrievalError::ForestNotFound(_)) => Ok(()),
            Err(e) => Err(UserCreationError::UserRetrievalError(e)),
        }?;
        tracing::trace!("User does not exist yet, creating new one");
        let crypto_identity = match input {
            CreateUserInput::Mnemonic(mnemonic) => Identity::try_from(mnemonic.as_ref())?,
            CreateUserInput::Entropy(entropy) => Identity::try_from(entropy.as_slice())?,
        };
        let master_identity = MasterIdentity::new(Some(crypto_identity));
        let default_forest_identity = master_identity
            .create_forest_identity(0)
            .map_err(UserCreationError::ForestIdentityCreationError)?;
        let device_identity = master_identity.create_device_identity(device_name.clone());

        let forest = self.catlib_service.add_forest(
            &default_forest_identity,
            &device_identity,
            ForestMetaData::new(vec![DeviceMetadata {
                name: device_name.clone(),
                pubkey: device_identity.get_public_key(),
            }]),
        )?;

        tracing::trace!("saving identities to lss");
        let forest_locked = forest.lock().expect("Poisoned Mutex");
        self.lss_service.save_forest_uuid(&*forest_locked)?;

        self.lss_service.save_identity(&default_forest_identity)?;
        self.lss_service.save_identity(&device_identity)?;

        Ok(CargoUser::new(
            device_name.clone(),
            vec![device_name],
            forest.clone(),
            self.catlib_service.clone(),
            &self.fsa_config,
            self.container_manager.clone(),
        ))
    }

    /// Retrieves default forest keypair from LSS and then basing on that reads User metadata from CatLib.
    /// Result is presented in from of [`crate::api::user::CargoUser`].
    ///
    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) fn get_user(&self) -> Result<Option<CargoUser>, UserRetrievalError> {
        let default_forest_uuid = self.get_default_forest_uuid()?;

        match self.catlib_service.get_forest(&default_forest_uuid) {
            Ok(forest) => {
                let user_metadata: ForestMetaData = serde_json::from_slice(
                    &forest
                        .lock()
                        .expect("Poisoned Mutex")
                        .data()
                        .map_err(UserRetrievalError::CatlibError)?,
                )
                .map_err(|e| {
                    CatlibError::Generic(format!(
                        "Could not parse forest data retrieved from Catlib: {e}"
                    ))
                })?;

                let device_identity = self
                    .lss_service
                    .get_this_device_identity()?
                    .ok_or(UserRetrievalError::DeviceMetadataNotFound)?;

                match user_metadata.get_device_metadata(device_identity.get_public_key()) {
                    Some(device_metadata) => Ok(Some(CargoUser::new(
                        device_metadata.name.clone(),
                        user_metadata.devices().map(|dm| dm.name.clone()).collect(),
                        forest,
                        self.catlib_service.clone(),
                        &self.fsa_config,
                        self.container_manager.clone(),
                    ))),
                    None => Err(UserRetrievalError::DeviceMetadataNotFound),
                }
            }
            Err(CatlibError::NoRecordsFound) => Ok(None),
            Err(err) => Err(UserRetrievalError::CatlibError(err)),
        }
    }

    /// Retrieves default forest uuid from LSS
    ///
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_default_forest_uuid(&self) -> Result<Uuid, UserRetrievalError> {
        tracing::debug!("searching for user");
        let forest_identity = self
            .lss_service
            .get_default_forest_identity()?
            .ok_or_else(|| {
                UserRetrievalError::ForestNotFound("Forest identity keypair not found".to_owned())
            })?;

        self.lss_service
            .get_forest_uuid_by_identity(&forest_identity)?
            .ok_or_else(|| UserRetrievalError::ForestNotFound("Forest uuid not found".to_owned()))
    }
}
