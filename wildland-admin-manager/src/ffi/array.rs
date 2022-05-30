///
/// Array type is very similar to RcRef, but it wraps
/// the vector structure and prepares the interface
/// to work with arrays.
///
///
use super::rcref::RcRef;

#[allow(dead_code)]
pub struct Array<T>(Vec<RcRef<T>>);
impl<T> Array<T> {
    #[allow(dead_code)]
    pub fn new_boxed(arr: Vec<RcRef<T>>) -> Box<Array<T>> {
        Box::new(Array(arr))
    }

    #[allow(dead_code)]
    pub fn at(&self, elem: usize) -> Box<RcRef<T>> {
        Box::new(self.0[elem].clone())
    }

    #[allow(dead_code)]
    pub fn size(&self) -> usize {
        self.0.len()
    }
}
