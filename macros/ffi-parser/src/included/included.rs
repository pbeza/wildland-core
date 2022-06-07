use std::fmt::Debug;
use std::sync::{Arc, Mutex};


#[allow(dead_code)]
pub struct Array<T: Clone>(Vec<T>);
impl<T: Clone> Array<T> {
    #[allow(dead_code)]
    pub fn new_boxed(arr: Vec<T>) -> Box<Array<T>> {
        Box::new(Array(arr))
    }

    #[allow(dead_code)]
    pub fn at(&self, elem: usize) -> T {
        self.0[elem].clone()
    }

    #[allow(dead_code)]
    pub fn size(&self) -> usize {
        self.0.len()
    }
}




#[derive(Debug)]
pub struct Opt<T>(Option<T>);
impl<T: Clone> Opt<T> {
    pub fn is_some(&self) -> bool {
        self.0.is_some()
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
}
impl<T> From<Option<T>> for Opt<T> {
    fn from(opt: Option<T>) -> Self {
        Opt(opt)
    }
}


#[derive(Debug)]
pub struct Shared<T: ?Sized + Unpin>(pub Arc<T>);

impl<T: ?Sized> Clone for Shared<T> {
    fn clone(&self) -> Shared<T> {
        Shared(self.0.clone())
    }
}

// impl<T> From<Arc<T>> for Shared<T> {
//     fn from(sh: Arc<T>) -> Self {
//         Shared(sh)
//     }
// }
// impl<T> Into<Arc<T>> for Shared<T> {
//     fn into(self) -> Arc<T> {
//         self.0
//     }
// }


#[derive(Debug)]
pub struct Mut<T: ?Sized + Unpin>(pub Mutex<T>);
// impl<T> From<Mutex<T>> for Mut<T> {
//     fn from(sh: Arc<T>) -> Self {
//         Shared(sh)
//     }
// }
// impl<T: ?Sized> Into<Mutex<T>> for Mut<T> {
//     fn into(self) -> Arc<T> {
//         self.0
//     }
// }




pub struct Res<T, E>(pub Result<T, E>);
impl<T, E> From<Result<T, E>> for Res<T, E> {
    fn from(res: Result<T, E>) -> Self {
        Res(res)
    }
}
impl<T, E> Into<Result<T, E>> for Res<T, E> {
    fn into(self) -> Result<T, E> {
        self.0
    }
}
impl<T: Clone + Debug, E: Debug + Clone> Res<T, E> {
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
    pub fn boxed_unwrap_err(&self) -> Box<E> {
        Box::new(self.unwrap_err())
    }
    pub fn unwrap_err(&self) -> E {
        self.0.as_ref().unwrap_err().clone()
    }
}
