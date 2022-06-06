use self::result::Res;
use crate::api::SeedPhrase;

mod array;
#[cfg(feature = "bindings")]
pub mod cxx;
mod identity;
mod option;
mod result;
#[cfg(feature = "swift-bindings")]
pub mod swift;

type SeedPhraseResult = Res<SeedPhrase>;
type EmptyResult = Res<()>;
