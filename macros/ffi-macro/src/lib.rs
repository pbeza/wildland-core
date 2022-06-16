extern crate proc_macro;
use ffi_parser::BindingModule;
use proc_macro::TokenStream;

///
/// # Example Rust module that can be translated:
///
/// ```rust
/// mod our_ffi_module {
///     use ffi_macro::binding_wrapper;
///     use std::sync::{Arc, Mutex};
///     
///     
///     // Define Error type and `()` type.
///     type ErrorType = String;
///     type VoidType = ();
///     
///     pub trait SomeTrait: std::fmt::Debug {
///         fn some_trait_method(&self);
///     }
///     
///     #[derive(Clone, Debug)]
///     pub struct Foo(u32);
///     impl SomeTrait for Foo {
///         fn some_trait_method(&self) {
///         }
///     }
///     
///     #[derive(Clone, Debug)]
///     pub struct CustomType(u32);
///     impl CustomType {
///         pub fn return_result_with_dynamic_type(&self) -> Result<Arc<Mutex<dyn SomeTrait>>, ErrorType> {
///             Ok(Arc::new(Mutex::new(Foo(10u32))))
///         }
///         pub fn return_another_custom_type(&self) -> AnotherCustomType {
///             AnotherCustomType(20u64)
///         }
///     }
///     
///     #[derive(Clone, Debug)]
///     pub struct AnotherCustomType(u64);
///     impl AnotherCustomType {
///         pub fn take_primitive_type_and_return_primitive_type(&self, a: u32) -> String {
///             "Result".to_owned()
///         }
///     }
///     
///     #[binding_wrapper]
///     mod ffi {
///         use super::SomeTrait;
///         extern "Rust" {
///             type CustomType;
///             fn return_result_with_dynamic_type(self: &CustomType) -> Result<Arc<Mutex<dyn SomeTrait>>>;
///             fn return_another_custom_type(self: &CustomType) -> AnotherCustomType;
///     
///             type AnotherCustomType;
///             fn take_primitive_type_and_return_primitive_type(self: &AnotherCustomType, a: u32) -> String;        
///             
///             fn some_trait_method(self: &Arc<Mutex<dyn SomeTrait>>);
///             type ErrorType;
///         }
///     }
/// }
/// ```
///
#[proc_macro_attribute]
pub fn binding_wrapper(_attr: TokenStream, input: TokenStream) -> TokenStream {
    BindingModule::parse(input.into())
        .unwrap()
        .get_tokens("swift_ffi")
        .into()
}
