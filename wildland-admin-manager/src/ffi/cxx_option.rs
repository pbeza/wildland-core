#[derive(Debug)]
pub struct CxxOption<T>(Option<T>);

impl<T> CxxOption<T> {
    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }

    pub fn unwrap(&mut self) -> &mut T {
        self.0.as_mut().unwrap()
    }
}

impl<T> From<Option<T>> for CxxOption<T> {
    fn from(opt: Option<T>) -> Self {
        CxxOption(opt)
    }
}
