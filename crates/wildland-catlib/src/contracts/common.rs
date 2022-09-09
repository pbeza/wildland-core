use super::CatlibResult;

pub(crate) trait Model {
    fn delete(&mut self) -> CatlibResult<()>;
    fn save(&mut self) -> CatlibResult<()>;
}
