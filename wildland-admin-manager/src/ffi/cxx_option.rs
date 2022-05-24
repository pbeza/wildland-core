pub struct CxxOption<T>(pub Option<T>);

impl<'a, T: std::fmt::Debug> CxxOption<T> {
    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }

    pub fn unwrap_mut(&mut self) -> &mut T {
        self.0.as_mut().unwrap()
    }
}

impl<'a, T> From<Option<T>> for CxxOption<T> {
    fn from(opt: Option<T>) -> Self {
        CxxOption(opt)
    }
}
