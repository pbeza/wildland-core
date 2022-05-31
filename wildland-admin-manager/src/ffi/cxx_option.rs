#[derive(Debug)]
pub struct CxxOption<T>(Option<T>);

impl<T> CxxOption<T> {
    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }

    // SWIG treat all references as mutable so there is no need to provide many unwrap methods
    // like e.g. unwrap for &ref and unwrap_mut for &mut ref
    // In C++ though, there is no possibility to obtain mutable reference without additional method
    pub fn unwrap(&self) -> &T {
        self.0.as_ref().unwrap()
    }
}

impl<T> From<Option<T>> for CxxOption<T> {
    fn from(opt: Option<T>) -> Self {
        CxxOption(opt)
    }
}
