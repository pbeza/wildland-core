extern crate proc_macro;
use ffi_parser::BindingModule;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

#[proc_macro_attribute]
pub fn binding_wrapper(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut tokens = TokenStream2::new();
    let parsed_bindings_cxx = BindingModule::parse_cxx(input.clone().into()).unwrap();
    tokens.extend(parsed_bindings_cxx.get_tokens("cxx_ffi"));
    let parsed_bindings_swift = BindingModule::parse_swift(input.into()).unwrap();
    tokens.extend(parsed_bindings_swift.get_tokens("swift_ffi"));
    tokens.into()
}
