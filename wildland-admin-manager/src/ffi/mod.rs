use crate::admin_manager::{AdminManager, Identity};
use thiserror::Error;
use wildland_admin_manager_api::{AdminManager as AMA, AdminManagerError, SeedPhraseWords};

type AdminManagerType = AdminManager<Identity>;

#[derive(Error)]
enum CxxResult<T> {
    Ok(T),
    Err(AdminManagerError),
}

impl<T: Clone + std::fmt::Debug> CxxResult<T> {
    fn is_ok(&self) -> bool {
        match self {
            CxxResult::Ok(_) => true,
            CxxResult::Err(_) => false,
        }
    }

    fn unwrap(&self) -> &T {
        match self {
            CxxResult::Ok(t) => t,
            CxxResult::Err(e) => panic!("Panicked while trying to unwrap error: {e}"),
        }
    }

    fn unwrap_err(&self) -> Box<CxxAdminManagerError> {
        match self {
            CxxResult::Ok(t) => {
                panic!("Panicked while trying to unwrap_err with valid value: {t:?}")
            }
            CxxResult::Err(e) => Box::new(CxxAdminManagerError(e)),
        }
    }
}

type SeedPhraseResult = CxxResult<SeedPhraseWords>;

struct CxxAdminManagerError<'a>(&'a AdminManagerError);

fn create_admin_manager() -> Box<AdminManagerType> {
    Box::new(AdminManager::<Identity>::default())
}

fn create_seed_phrase() -> Box<SeedPhraseResult> {
    Box::new(CxxResult::Ok(
        AdminManager::<Identity>::create_seed_phrase().unwrap(),
    ))
}

#[cxx::bridge(namespace = "cargo::api")]
mod api {
    extern "Rust" {
        type SeedPhraseResult;
        type AdminManagerType;
        type CxxAdminManagerError<'a>;

        fn is_ok(self: &SeedPhraseResult) -> bool;
        fn unwrap(self: &SeedPhraseResult) -> &[String; 12];
        fn unwrap_err(self: &SeedPhraseResult) -> Box<CxxAdminManagerError>;

        fn create_admin_manager() -> Box<AdminManagerType>;
        fn create_seed_phrase() -> Box<SeedPhraseResult>;
    }
}
