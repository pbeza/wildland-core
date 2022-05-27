use self::result::Res;
use crate::api::SeedPhrase;

mod array;
pub mod cxx;
mod option;
mod rcref;
mod result;
pub mod swift;

type SeedPhraseResult = Res<SeedPhrase>;
