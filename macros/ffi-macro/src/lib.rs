extern crate proc_macro;
use ffi_parser::BindingModule;
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn binding_wrapper(_attr: TokenStream, input: TokenStream) -> TokenStream {
    BindingModule::parse(input.into())
        .unwrap()
        .get_tokens("swift_ffi")
        .into()
}
