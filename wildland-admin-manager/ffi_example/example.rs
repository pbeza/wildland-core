/// FFI example file


struct CustomStruct;
impl CustomStruct {
    fn print_foo(&self) {
        println!("Foo from CustomStruct")
    }
}

//
// All templated types have to be manually instantiated (cxx.rs constraint)
//
type CustomStruct = CustomStruct;
type RcRefCustomStruct = RcRef<CustomStruct>;
type ArrayCustomStruct = Array<CustomStruct>;

//
// The module with functions and types declarations
// visible in wildland's clients.
//
#[cxx::bridge(namespace = "wildland")]
mod ffi_cxx {
    extern "Rust" {
        // CustomStruct declaration
        type CustomStruct;
        fn print_foo(self: &CustomStruct);

        // RcRef<CustomStruct> declaration
        type RcRefCustomStruct;
        fn deref(self: &RcRefCustomStruct) -> &CustomStruct;

        // Array<CustomStruct> declaration
        type ArrayCustomStruct;
        fn at(self: &ArrayCustomStruct, elem: usize) -> Box<RcRefCustomStruct>;
        fn size(self: &ArrayCustomStruct) -> usize;

        // Static functions declarations
        fn return_string() -> String;
        fn return_vec_string() -> Vec<String>;
        fn return_vec_u8() -> Vec<u8>;
        fn return_u8() -> u8;
        fn print_args(a: Vec<String>, b: Vec<u8>, c: u8, d: String);

        // TODO: this is the only difference between cxx and swift for now:
        fn get_admin_instances_vector_ref() -> Box<ArrayCustomStruct>;
        fn get_admin_instance_ref() -> Box<RcRefCustomStruct>;
    }
}

#[swift_bridge::bridge]
mod ffi_swift {
    extern "Rust" {
        type CustomStructError;

        // CustomStruct declaration
        type CustomStruct;
        fn print_foo(self: &CustomStruct);

        // RcRef<CustomStruct> declaration
        type RcRefCustomStruct;
        fn deref(self: &RcRefCustomStruct) -> &CustomStruct;

        // Array<CustomStruct> declaration
        type ArrayCustomStruct;
        fn at(self: &ArrayCustomStruct, elem: usize) -> RcRefCustomStruct;
        fn size(self: &ArrayCustomStruct) -> usize;

        // Static functions declarations
        fn return_string() -> String;
        fn return_vec_string() -> Vec<String>;
        fn return_vec_u8() -> Vec<u8>;
        fn return_u8() -> u8;
        fn print_args(a: Vec<String>, b: Vec<u8>, c: u8, d: String);

        // TODO: this is the only difference between cxx and swift for now:
        fn get_admin_instance() -> RcRefCustomStruct;
        fn get_admin_instances_vector() -> Vec<RcRefCustomStruct>;
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

pub fn get_admin_instance() -> RcRefCustomStruct {
    RcRef::new(CustomStruct::default())
}

pub fn get_admin_instance_ref() -> Box<RcRefCustomStruct> {
    RcRef::new_boxed(CustomStruct::default())
}

pub fn get_admin_instances_vector() -> Vec<RcRefCustomStruct> {
    vec![RcRef::new(CustomStruct::default())]
}

pub fn get_admin_instances_vector_ref() -> Box<ArrayCustomStruct> {
    Array::new_boxed(vec![RcRef::new(CustomStruct::default())])
}
