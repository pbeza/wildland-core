use crate::admin_manager;

type AdminManager = admin_manager::AdminManager<admin_manager::Identity>;

#[cxx::bridge(namespace = "wildland::adminmanager")]
mod ffi {
    extern "Rust" {
        type AdminManager;
        fn print_foo(self: &AdminManager);
        fn get_admin() -> &'static AdminManager;

        fn return_string() -> String;
        fn return_vec_string() -> Vec<String>;
        fn return_vec_u8() -> Vec<u8>;
        fn return_u8() -> u8;
        fn print_args(a: Vec<String>, b: Vec<u8>, c: u8, d: String);

        // type Adam;
        // fn give_me_adams() -> Vec<&'static Adam>;
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

// struct Adam {}

// fn give_me_adams() -> Vec<&'static Adam> {
//     vec![]
// }
