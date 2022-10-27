#[derive(Debug, Clone)]
pub struct Storage {}

impl Storage {
    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub fn stringify(&self) -> String {
        todo!()
    }
}
