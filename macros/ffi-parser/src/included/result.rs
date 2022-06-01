use std::fmt::Debug;

pub struct Res<T>(pub ResultFfi<T>);

impl<T> From<ResultFfi<T>> for Res<T> {
    fn from(res: ResultFfi<T>) -> Self {
        Res(res)
    }
}

impl<T: Clone + Debug> Res<T> {
    pub fn is_ok(&self) -> bool {
        self.0.is_ok()
    }

    // SWIG treat all references as mutable so there is no need to provide many unwrap methods
    // like e.g. unwrap for &ref and unwrap_mut for &mut ref
    // In C++ though, there is no possibility to obtain mutable reference without additional method
    pub fn boxed_unwrap(&self) -> Box<T> {
        Box::new(self.unwrap())
    }
    pub fn unwrap(&self) -> T {
        self.0.as_ref().unwrap().clone()
    }

    pub fn boxed_unwrap_err(&self) -> Box<ResultFfiError> {
        Box::new(self.unwrap_err())
    }
    pub fn unwrap_err(&self) -> ResultFfiError {
        self.0.as_ref().unwrap_err().clone()
    }
}
