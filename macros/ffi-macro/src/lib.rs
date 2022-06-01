extern crate proc_macro;
use ffi_parser::BindingModule;
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn binding_wrapper(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let parsed_bindings = BindingModule::parse(input.clone().into()).unwrap();
    parsed_bindings.get_tokens().into()
}
