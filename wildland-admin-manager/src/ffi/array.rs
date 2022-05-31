///
/// Array wraps the vector structure and prepares the interface to work with arrays.
///

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
