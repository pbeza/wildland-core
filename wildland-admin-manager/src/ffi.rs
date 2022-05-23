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

type RcRefAdminManager = RcRef<AdminManager>;
type AdminManager = admin_manager::AdminManager<admin_manager::Identity>;

#[cxx::bridge(namespace = "wildland::adminmanager")]
mod ffi_definition {
    extern "Rust" {
        type RcRefAdminManager;
        type AdminManager;
        unsafe fn deref(self: &RcRefAdminManager) -> &AdminManager;

        fn print_foo(self: &AdminManager);
        fn get_admin() -> &'static AdminManager;
        fn return_string() -> String;
        fn return_vec_string() -> Vec<String>;
        fn return_vec_u8() -> Vec<u8>;
        fn return_u8() -> u8;
        fn print_args(a: Vec<String>, b: Vec<u8>, c: u8, d: String);

        // There's a problem with lack of constructors for AdminManagerRef
        // fn return_vec_custom_struct_rc() -> Vec<AdminManagerRef>;
        // CXX.rs doesn't support the following:
        // fn return_vec_custom_struct_rc() -> Vec<Box<AdminManagerRef>>;

        fn return_rc() -> Box<RcRefAdminManager>;
    }
}

lazy_static::lazy_static! {
    pub static ref GLOBAL_ADMIN_MANAGER: AdminManager = AdminManager::default();
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

pub fn get_admin() -> &'static AdminManager {
    &GLOBAL_ADMIN_MANAGER
}

pub fn print_args(a: Vec<String>, b: Vec<u8>, c: u8, d: String) {
    println!("{:?}", a);
    println!("{:?}", b);
    println!("{:?}", c);
    println!("{:?}", d);
}

// pub fn return_vec_custom_struct_rc() -> Vec<AdminManagerRef> {
//     vec![]
// }

pub fn return_rc() -> Box<RcRefAdminManager> {
    RcRef::new_boxed(AdminManager::default())
}
