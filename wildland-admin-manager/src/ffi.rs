use crate::admin_manager;
use std::sync::Arc;

pub struct RcRef<T>(Arc<T>);
impl<T> RcRef<T> {
    fn new_boxed(obj: T) -> Box<RcRef<T>> {
        Box::new(RcRef::<T>(Arc::new(obj)))
    }

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> Drop for RcRef<T> {
    fn drop(&mut self) {
        //TODO: add logging handler
        println!("DEBUG: Droping RcRef")
    }
}

pub struct Array<T>(Arc<Vec<T>>);
impl<T> Array<T> {
    pub fn new_boxed(arr: Vec<T>) -> Box<Array<T>> {
        Box::new(Array(Arc::new(arr)))
    }

    pub fn at(&self, elem: usize) -> &T {
        &self.0[elem]
    }

    pub fn size(&self) -> usize {
        self.0.len()
    }
}

type RcRefAdminManager = RcRef<AdminManager>;
type ArrayAdminManager = Array<AdminManager>;
type AdminManager = admin_manager::AdminManager<admin_manager::Identity>;


#[cxx::bridge(namespace = "wildland")]
mod ffi_definition {
    extern "Rust" {
        type AdminManager;

        type RcRefAdminManager;
        fn get_admin_instance() -> Box<RcRefAdminManager>;
        fn deref(self: &RcRefAdminManager) -> &AdminManager;

        type ArrayAdminManager;
        fn get_admin_instances_vector() -> Box<ArrayAdminManager>;
        fn at(self: &ArrayAdminManager, elem: usize) -> &AdminManager;
        fn size(self: &ArrayAdminManager) -> usize;

        fn print_foo(self: &AdminManager);
        fn return_string() -> String;
        fn return_vec_string() -> Vec<String>;
        fn return_vec_u8() -> Vec<u8>;
        fn return_u8() -> u8;
        fn print_args(a: Vec<String>, b: Vec<u8>, c: u8, d: String);
    }
}

pub fn return_string() -> String {
    String::from("Returned String")
}

pub fn return_vec_string() -> Vec<String> {
    vec!["First".into(), "Second".into()]
}

pub fn return_vec_u8() -> Vec<u8> {
    vec![10, 20]
}

pub fn return_u8() -> u8 {
    255
}

pub fn print_args(a: Vec<String>, b: Vec<u8>, c: u8, d: String) {
    println!("{:?}", a);
    println!("{:?}", b);
    println!("{:?}", c);
    println!("{:?}", d);
}

pub fn get_admin_instance() -> Box<RcRefAdminManager> {
    RcRef::new_boxed(AdminManager::default())
}

pub fn get_admin_instances_vector() -> Box<ArrayAdminManager> {
    Array::new_boxed(vec![AdminManager::default()])
}
