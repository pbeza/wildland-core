#[derive(Debug)]
pub struct Opt<T>(Option<T>);

impl<T> Opt<T> {
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

impl<T> From<Option<T>> for Opt<T> {
    fn from(opt: Option<T>) -> Self {
        Opt(opt)
    }
}
