pub struct CxxRefOption<'a, T>(pub &'a mut Option<T>);

impl<'a, T: Clone + std::fmt::Debug> CxxRefOption<'a, T> {
    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }

    pub fn unwrap(&mut self) -> &mut T {
        self.0.as_mut().unwrap()
    }
}

impl<'a, T> From<&'a mut Option<T>> for CxxRefOption<'a, T> {
    fn from(opt: &'a mut Option<T>) -> Self {
        CxxRefOption(opt)
    }
}
