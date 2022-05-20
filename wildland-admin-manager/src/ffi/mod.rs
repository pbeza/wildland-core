use crate::{
    admin_manager::{AdminManager, Identity},
    api::{AdminManager as AdminManagerApi, AdminManagerResult, SeedPhrase},
};
use std::fmt::Debug;

type AdminManagerType = AdminManager<Identity>;

impl<T: Clone + std::fmt::Debug> CxxResult<T> {
    fn is_ok(&self) -> bool {
        self.0.is_ok()
    }

    fn unwrap(&self) -> &T {
        self.0.as_ref().unwrap()
    }

    // fn unwrap_err(&self) -> Box<CxxAdminManagerError> {
    //     match self {
    //         CxxResult::Ok(t) => {
    //             panic!("Panicked while trying to unwrap_err with valid value: {t:?}")
    //         }
    //         CxxResult::Err(e) => Box::new(CxxAdminManagerError(e)),
    //     }
    // }
}

#[derive(Debug)]
struct CxxResult<T: Debug>(AdminManagerResult<T>);

impl<T: Debug> From<AdminManagerResult<T>> for CxxResult<T> {
    fn from(res: AdminManagerResult<T>) -> Self {
        CxxResult(res)
    }
}

type SeedPhraseResult = CxxResult<SeedPhrase>;

fn create_admin_manager() -> Box<AdminManagerType> {
    Box::new(AdminManager::<Identity>::default())
}

fn create_seed_phrase() -> Box<SeedPhraseResult> {
    Box::new(AdminManager::<Identity>::create_seed_phrase().into())
}

#[cxx::bridge(namespace = "cargo::api")]
mod api {
    extern "Rust" {
        type SeedPhraseResult;
        type SeedPhrase;

        type AdminManagerType;
        // type CxxAdminManagerError<'a>;

        fn is_ok(self: &SeedPhraseResult) -> bool;
        fn unwrap(self: &SeedPhraseResult) -> &SeedPhrase;
        fn get_string(self: &SeedPhrase) -> String;
        // fn unwrap_err(self: &SeedPhraseResult) -> Box<CxxAdminManagerError>;

        fn create_admin_manager() -> Box<AdminManagerType>;
        fn create_seed_phrase() -> Box<SeedPhraseResult>;
    }
}
