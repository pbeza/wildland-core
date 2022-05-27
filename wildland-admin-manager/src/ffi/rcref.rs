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

#[derive(Debug)]
pub struct RcRef<T: ?Sized>(Arc<T>);
impl<T: ?Sized> RcRef<T> {
    #[allow(dead_code)]
    pub fn from_arc(obj: Arc<T>) -> RcRef<T> {
        RcRef::<T>(obj)
    }

    pub fn get_mut(&mut self) -> &mut T {
        unsafe { Arc::<T>::get_mut_unchecked(&mut self.0) }
    }

    pub fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> Clone for RcRef<T> {
    fn clone(&self) -> RcRef<T> {
        RcRef(self.0.clone())
    }
}

impl<T: ?Sized> Drop for RcRef<T> {
    fn drop(&mut self) {
        //TODO: add logging handler
        println!("DEBUG: Droping RcRef")
    }
}
