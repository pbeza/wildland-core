///
/// RcRef is used as a shared pointer that can be used in languages
/// supported by `SWIG`. The mentioned tool takes care of garbage
/// collectors handling.
///
/// When the target client delete RcRef object, the reference count
/// will be decreased. The pointee object is deleted only if there's
/// no other reference (on both sides - Rust and the target lang)
/// available.
///
use std::sync::Arc;

pub struct RcRef<T>(Arc<T>);
impl<T> RcRef<T> {
    #[allow(dead_code)]
    pub fn new(obj: T) -> RcRef<T> {
        RcRef::<T>(Arc::new(obj))
    }

    #[allow(dead_code)]
    fn new_boxed(obj: T) -> Box<RcRef<T>> {
        Box::new(RcRef::<T>(Arc::new(obj)))
    }

    #[allow(dead_code)]
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> Clone for RcRef<T> {
    fn clone(&self) -> RcRef<T> {
        RcRef(self.0.clone())
    }
}

impl<T> Drop for RcRef<T> {
    fn drop(&mut self) {
        //TODO: add logging handler
        println!("DEBUG: Droping RcRef")
    }
}
