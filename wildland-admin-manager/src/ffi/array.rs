///
/// Array type is very similar to RcRef, but it wraps
/// the vector structure and prepares the interface
/// to work with arrays.
///
/// 

use super::rcref::RcRef;

pub struct Array<T>(Vec<RcRef<T>>);
impl<T> Array<T> {
    pub fn new_boxed(arr: Vec<RcRef<T>>) -> Box<Array<T>> {
        Box::new(Array(arr))
    }

    pub fn at(&self, elem: usize) -> Box<RcRef<T>> {
        Box::new(self.0[elem].clone())
    }

    pub fn size(&self) -> usize {
        self.0.len()
    }
}
