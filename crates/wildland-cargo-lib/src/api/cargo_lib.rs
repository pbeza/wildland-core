#[cfg(test)]
use crate::test_utils::MockLssService as LssService;
use crate::{
    api::{config::CargoConfig, user::UserApi},
    errors::{SingleErrVariantResult, SingleVariantError},
    logging,
    user::UserService,
};
use std::{
    mem::MaybeUninit,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};
use thiserror::Error;
use wildland_corex::LocalSecureStorage;
#[cfg(not(test))]
use wildland_corex::LssService;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
#[error("CargoLib creation error: {0}")]
pub struct CargoLibCreationError(pub String);

static INITIALIZED: AtomicBool = AtomicBool::new(false);

type SharedCargoLib = Arc<Mutex<CargoLib>>;
static mut CARGO_LIB: MaybeUninit<SharedCargoLib> = MaybeUninit::uninit();

/// Structure aggregating and exposing public API of CargoLib library.
/// All functionalities are exposed to application side through this structure.
///
/// It can be created with [`create_cargo_lib`] function.
///
#[derive(Clone)]
pub struct CargoLib {
    user_api: UserApi,
}

impl CargoLib {
    pub(crate) fn new(user_api: UserApi) -> Self {
        Self { user_api }
    }

    /// Returns structure aggregating API for user management
    #[tracing::instrument(skip(self))]
    pub fn user_api(&self) -> UserApi {
        self.user_api.clone()
    }
}

/// [`CargoLib`] initializer.
///
/// Underlying structure is created only once, subsequent call will return handle to the same structure.
///
/// It requires the following arguments:
/// - lss: some type implementing [`LocalSecureStorage`] trait. It is usually provided by the native
/// to a target platform language. It is assumed that a lss reference should be valid for a whole
/// program execution (static lifetime).
/// - cfg: [`CargoConfig`] structure with config variables (logger, endpoints, etc.)
///
/// CargoLib expects to get references with static lifetimes so it is important not to inline
/// objects (e.g. LSS) initialization along with createCargoLib call.
///
/// ```
/// # use wildland_corex::{LocalSecureStorage, LssResult};
/// # use wildland_cargo_lib::api::{config::CargoConfig, cargo_lib::create_cargo_lib};
/// # use tracing::Level;
/// #
/// struct TestLss{};
///
/// impl LocalSecureStorage for TestLss {
/// // ...implementation here
/// #    fn insert(&self, key: String, value: Vec<u8>) -> LssResult<Option<Vec<u8>>>{todo!()}
/// #    fn get(&self, key: String) -> LssResult<Option<Vec<u8>>>{todo!()}
/// #    fn contains_key(&self, key: String) -> LssResult<bool>{todo!()}
/// #    fn keys(&self) -> LssResult<Vec<String>>{todo!()}
/// #    fn remove(&self, key: String) -> LssResult<Option<Vec<u8>>>{todo!()}
/// #    fn len(&self) -> LssResult<usize>{todo!()}
/// #    fn is_empty(&self) -> LssResult<bool>{todo!()}
/// }
///
/// let lss = TestLss{};
///
/// let cfg = CargoConfig{
///     log_level: Level::DEBUG,
///     log_file: None,
/// };
///
/// let lss: &'static TestLss = unsafe { std::mem::transmute(&lss) };
/// let cargo_lib = create_cargo_lib(lss, cfg);
/// ```
pub fn create_cargo_lib(
    lss: &'static dyn LocalSecureStorage,
    cfg: CargoConfig,
) -> SingleErrVariantResult<SharedCargoLib, CargoLibCreationError> {
    if !INITIALIZED.load(Ordering::Relaxed) {
        INITIALIZED.store(true, Ordering::Relaxed);

        logging::init_subscriber(cfg.log_level, cfg.log_file)
            .map_err(|e| SingleVariantError::Failure(CargoLibCreationError(e)))?;

        let cargo_lib = Arc::new(Mutex::new(CargoLib::new(UserApi::new(UserService::new(
            LssService::new(lss),
        )))));

        unsafe {
            CARGO_LIB.write(cargo_lib);
        }
    }
    unsafe { Ok(CARGO_LIB.assume_init_ref().clone()) }
}
