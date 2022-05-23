use crate::admin_manager;
use std::boxed::Box;
use std::sync::Arc;

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
pub struct RcRef<T>(Arc<T>);
impl<T> RcRef<T> {
    fn new(obj: T) -> RcRef<T> {
        RcRef::<T>(Arc::new(obj))
    }

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

///
/// Array type is very similar to RcRef, but it wraps
/// the vector structure and prepares the interface
/// to work with arrays.
///
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

//
// All templated types have to be manually instantiated (cxx.rs constraint)
//
type AdminManager = admin_manager::AdminManager<admin_manager::Identity>;
type RcRefAdminManager = RcRef<AdminManager>;
type ArrayAdminManager = Array<AdminManager>;

//
// The module with functions and types declarations
// visible in wildland's clients.
//
#[cfg(feature = "cxx_binding")]
#[cxx::bridge(namespace = "wildland")]
mod ffi_definition {
    extern "Rust" {
        // AdminManager implementation
        type AdminManager;
        fn print_foo(self: &AdminManager);

        // RcRef<AdminManager> declarations
        type RcRefAdminManager;
        fn deref(self: &RcRefAdminManager) -> &AdminManager;

        // Array<AdminManager> declarations
        type ArrayAdminManager;
        fn at(self: &ArrayAdminManager, elem: usize) -> &AdminManager;
        fn size(self: &ArrayAdminManager) -> usize;

        // Static functions declarations
        fn return_string() -> String;
        fn return_vec_string() -> Vec<String>;
        fn return_vec_u8() -> Vec<u8>;
        fn return_u8() -> u8;
        fn print_args(a: Vec<String>, b: Vec<u8>, c: u8, d: String);
        fn get_admin_instances_vector() -> Box<ArrayAdminManager>;
        fn get_admin_instance() -> Box<RcRefAdminManager>;
    }
}

#[cfg(feature = "swift_binding")]
#[swift_bridge::bridge]
mod ffi_definition {
    extern "Rust" {
        // AdminManager implementation
        type AdminManager;
        fn print_foo(self: &AdminManager);

        // RcRef<AdminManager> declarations
        type RcRefAdminManager;
        fn deref(self: &RcRefAdminManager) -> &AdminManager;

        // Static functions declarations
        fn return_string() -> String;
        fn return_vec_string() -> Vec<String>;
        fn return_vec_u8() -> Vec<u8>;
        fn return_u8() -> u8;
        fn print_args(a: Vec<String>, b: Vec<u8>, c: u8, d: String);
        fn get_admin_instance() -> RcRefAdminManager;
        fn get_admin_instances_vector() -> Vec<RcRefAdminManager>;
    }
}

//
// Implementations of static functions available for the client app
//

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

#[cfg(feature = "cxx_binding")]
pub fn get_admin_instance() -> Box<RcRefAdminManager> {
    RcRef::new_boxed(AdminManager::default())
}

#[cfg(feature = "swift_binding")]
pub fn get_admin_instance() -> RcRefAdminManager {
    RcRef::new(AdminManager::default())
}

#[cfg(feature = "cxx_binding")]
pub fn get_admin_instances_vector() -> Box<ArrayAdminManager> {
    Array::new_boxed(vec![AdminManager::default()])
}

#[cfg(feature = "swift_binding")]
pub fn get_admin_instances_vector() -> Vec<RcRefAdminManager> {
    vec![RcRef::new(AdminManager::default())]
}
