pub struct CxxRefOption<'a, T>(pub &'a Option<T>);

impl<'a, T: Clone + std::fmt::Debug> CxxRefOption<'a, T> {
    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }

    pub fn unwrap(&self) -> &T {
        self.0.as_ref().unwrap()
    }
}

impl<'a, T> From<&'a Option<T>> for CxxRefOption<'a, T> {
    fn from(opt: &'a Option<T>) -> Self {
        CxxRefOption(opt)
    }
}
