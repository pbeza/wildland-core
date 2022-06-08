use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    parse_quote, Block, ForeignItem, ForeignItemType, Item, ItemFn, ItemForeignMod, ItemMod, Type,
};

use crate::{binding_types::*, function_transform::Transformer};

#[derive(Default)]
pub struct BindingModule {
    transformer: Transformer,
    module: Option<ItemMod>,
    generated: TokenStream,
}

impl BindingModule {
    fn get_vec_of_extern_items_from_module(
        module: &mut ItemMod,
    ) -> Result<&mut Vec<ForeignItem>, String> {
        module
            .content
            .as_mut()
            .ok_or("The module is empty.".to_owned())?
            .1
            .iter_mut()
            .find_map(|module_item| match module_item {
                Item::ForeignMod(rust_module) => Some(&mut rust_module.items),
                _ => None,
            })
            .ok_or("Expected extern \"Rust\" in module definition.".to_owned())
    }

    fn generate_rust_wrappers_in_extern_mod(&mut self, boxed_result: bool) -> Result<(), String> {
        for wrapper in &self.transformer.rust_types_wrappers {
            let wrapper_name = &wrapper.wrapper_name;
            let original_type_name = &wrapper.original_type_name;
            let return_original_type_name: Type = if boxed_result {
                parse_quote! ( Box<#original_type_name> )
            } else {
                parse_quote! (#original_type_name)
            };
            let error_type_name: Type = if boxed_result {
                parse_quote!(Box<RustResultFfiError>)
            } else {
                parse_quote!(RustResultFfiError)
            };
            let tokens = match wrapper.typ {
                RustWrapperType::Result => quote! {
                    extern "Rust" {
                        type #wrapper_name;
                        fn unwrap(self: &#wrapper_name) -> #return_original_type_name;
                        fn unwrap_err(self: &#wrapper_name) -> #error_type_name;
                        fn is_ok(self: &#wrapper_name) -> bool;
                    }
                },
                RustWrapperType::Option => quote! {
                    extern "Rust" {
                        type #wrapper_name;
                        fn unwrap(self: &#wrapper_name) -> #return_original_type_name;
                        fn is_some(self: &#wrapper_name) -> bool;
                    }
                },
                RustWrapperType::Vector => {
                    if wrapper
                        .inner_type
                        .as_ref()
                        .expect("Vector has to have inner generic type")
                        .typ
                        != RustWrapperType::Primitive
                    {
                        quote! {
                            extern "Rust" {
                                type #wrapper_name;
                                fn at(self: &#wrapper_name) -> #return_original_type_name;
                                fn size(self: &#wrapper_name) -> usize;
                            }
                        }
                    } else {
                        quote!(
                            extern "Rust" {}
                        )
                    }
                }
                RustWrapperType::Arc => quote! {
                    extern "Rust" { type #wrapper_name; }
                },
                RustWrapperType::Custom => quote! {
                    extern "Rust" { type #wrapper_name; }
                },
                RustWrapperType::Primitive => quote! { extern "Rust" {} },
            };
            let module_module: ItemForeignMod = parse_quote!(#tokens);
            BindingModule::get_vec_of_extern_items_from_module(
                self.module.as_mut().ok_or("Module not found")?,
            )
            .map_err(|_| "Couldn't find extern \"Rust\"")?
            .extend(module_module.items.iter().cloned());
        }
        Ok(())
    }

    fn generate_rust_wrappers_definitions(&mut self, boxed_result: bool) {
        for wrapper in &self.transformer.rust_types_wrappers {
            let wrapper_name = &wrapper.wrapper_name;
            let original_type_name = &wrapper.original_type_name;
            let return_original_type_name: Type = if boxed_result {
                parse_quote! ( Box<#original_type_name> )
            } else {
                parse_quote! (#original_type_name)
            };
            let error_type_name: Type = if boxed_result {
                parse_quote!(Box<RustResultFfiError>)
            } else {
                parse_quote!(RustResultFfiError)
            };
            let tokens: TokenStream = match &wrapper.typ {
                RustWrapperType::Result => {
                    // SWIG treat all references as mutable so there is no need to provide many unwrap methods
                    // like e.g. unwrap for &ref and unwrap_mut for &mut ref
                    // In C++ though, there is no possibility to obtain mutable reference without additional method
                    quote! {
                        pub struct #wrapper_name(Result<#original_type_name, RustResultFfiError>);
                        impl #wrapper_name {
                            pub fn is_ok(&self) -> bool {
                                self.0.is_ok()
                            }
                            pub fn unwrap(&self) -> #return_original_type_name {
                                self.0.as_ref().unwrap().clone().into()
                            }
                            pub fn unwrap_err(&self) -> #error_type_name {
                                self.0.as_ref().unwrap_err().clone().into()
                            }
                        }
                    }
                    .into()
                }
                RustWrapperType::Option => quote! {
                    pub struct #wrapper_name(Option<#original_type_name>);
                    impl #wrapper_name {
                        pub fn is_some(&self) -> bool {
                            self.0.is_some()
                        }
                        pub fn unwrap(&self) -> #return_original_type_name {
                            self.0.as_ref().unwrap().clone().into()
                        }
                    }
                }
                .into(),
                RustWrapperType::Vector => if wrapper
                    .inner_type
                    .as_ref()
                    .expect("Vector has to have inner generic type")
                    .typ
                    != RustWrapperType::Primitive
                {
                    quote! {
                        pub struct #wrapper_name(Vec<#original_type_name>);
                        impl #wrapper_name {
                            pub fn at(&self, elem: usize) -> #return_original_type_name {
                                self.0[elem].clone()
                            }
                            pub fn size(&self) -> usize {
                                self.0.len()
                            }
                        }
                    }
                } else {
                    quote! {}
                }
                .into(),
                _ => quote! {}.into(),
            };
            self.generated.extend(tokens);
        }
    }

    fn generate_function_body(
        functions: &Vec<Function>,
        skip_first: bool,
        custom_self: Option<TokenStream>,
    ) -> Vec<ItemFn> {
        functions
            .iter()
            .map(|function| {
                let fn_name = &function.parsed_items.sig.ident;
                let args = function
                    .arguments
                    .iter()
                    .skip(skip_first as usize)
                    .map(|Arg { arg_name, .. }| quote! {  #arg_name.into() })
                    .collect::<Vec<_>>();
                let struct_name: TokenStream = if skip_first {
                    if let Some(custom_self) = &custom_self {
                        custom_self.clone()
                    } else {
                        quote! { self.0. }
                    }
                } else {
                    quote! { crate:: }
                };
                let fn_call = if let Some(wrapper) = &function.return_type {
                    let wrapper_name = &wrapper.wrapper_name;
                    let original_type_name = &wrapper.original_type_name;
                    match wrapper.typ {
                        RustWrapperType::Result => {
                            quote! {{
                                #wrapper_name(
                                    #struct_name #fn_name( #(#args),* )
                                        .map(|ok| #original_type_name::from(ok))
                                        .map_err(|err| RustResultFfiError::from(err))
                                ).into()
                            }}
                        }
                        RustWrapperType::Option => {
                            quote! {{
                                #wrapper_name(
                                    #struct_name #fn_name( #(#args),* )
                                        .map(|ok| #original_type_name::from(ok))
                                ).into()
                            }}
                        }
                        RustWrapperType::Vector => if wrapper
                        .inner_type
                        .as_ref()
                        .expect("Vector has to have inner generic type")
                        .typ
                        != RustWrapperType::Primitive
                        {
                            quote! {{ #wrapper_name(#struct_name #fn_name( #(#args),* )).into() }}
                        } else {
                            quote! {{ #struct_name #fn_name( #(#args),* ).into() }}
                        },
                        RustWrapperType::Custom => {
                            quote! {{ #wrapper_name(#struct_name #fn_name( #(#args),* )).into() }}
                        }
                        RustWrapperType::Arc => {
                            quote! {{ #struct_name #fn_name( #(#args),* ) }}
                        }
                        RustWrapperType::Primitive => {
                            quote! {{ #struct_name #fn_name( #(#args),* ) }}
                        }
                    }
                } else {
                    quote! {{ #struct_name #fn_name( #(#args),* ); }}
                };
                let block: Block = parse_quote!(#fn_call);
                ItemFn {
                    attrs: function.parsed_items.attrs.clone(),
                    vis: function.parsed_items.vis.clone(),
                    sig: function.parsed_items.sig.clone(),
                    block: block.into(),
                }
            })
            .collect()
    }

    fn generate_custom_types_wrappers(&mut self) {
        for (custom_type, functions) in &self.transformer.structures_wrappers {
            let wrapper_name = &custom_type.wrapper_name;
            let original_type_name = &custom_type.original_type_name;
            match custom_type.typ {
                RustWrapperType::Custom => {
                    let generated_functions =
                        BindingModule::generate_function_body(&functions, true, None);
                    let generated_custom_wrapper_types: TokenStream = quote! {
                        #[derive(Clone, Debug)]
                        pub struct #wrapper_name(#original_type_name);
                        impl #wrapper_name {
                            #(#generated_functions)*
                        }
                        impl From<#original_type_name> for #wrapper_name {
                            fn from(w: #original_type_name) -> #wrapper_name {
                                #wrapper_name(w)
                            }
                        }
                        impl<'a> Into<&'a #original_type_name> for &'a #wrapper_name {
                            fn into(self) -> &'a #original_type_name {
                                &self.0
                            }
                        }
                    }
                    .into();
                    self.generated.extend(generated_custom_wrapper_types);
                }
                RustWrapperType::Arc => {
                    let generated_functions = BindingModule::generate_function_body(
                        &functions,
                        true,
                        Some(quote! {self.0.lock().unwrap().}),
                    );
                    let generated_custom_wrapper_types: TokenStream = quote! {
                        #[derive(Clone, Debug)]
                        pub struct #wrapper_name(Arc<#original_type_name>);
                        impl #wrapper_name {
                            #(#generated_functions)*
                        }
                        impl From<Arc<#original_type_name>> for #wrapper_name {
                            fn from(w: Arc<#original_type_name>) -> #wrapper_name {
                                #wrapper_name(w)
                            }
                        }
                    }
                    .into();
                    self.generated.extend(generated_custom_wrapper_types);
                }
                _ => {}
            }
        }
    }

    fn generate_global_functions_wrappers(&mut self) {
        let generated_functions =
            BindingModule::generate_function_body(&self.transformer.global_functions, false, None);
        let generated_custom_wrapper_types = quote! { #(#generated_functions)* };
        self.generated.extend(generated_custom_wrapper_types);
    }

    //////////////////////////////////
    //////////////////////////////////
    //////////////////////////////////

    pub fn generate_swig_interface_file_from_cxx_module(&self) -> String {
        self.transformer
            .rust_types_wrappers
            .iter()
            .map(|key| match key.typ {
                RustWrapperType::Custom => format!(
                    "%template(Boxed{}) ::rust::cxxbridge1::Box<::{}>;\n",
                    key.wrapper_name, key.wrapper_name
                ),
                RustWrapperType::Vector => format!(
                    "%template(Vec{}) ::rust::cxxbridge1::Vec<::{}>;\n",
                    key.inner_type
                        .as_ref()
                        .expect("Vector has to have inner generic type")
                        .wrapper_name,
                    key.inner_type
                        .as_ref()
                        .expect("Vector has to have inner generic type")
                        .wrapper_name
                ),
                RustWrapperType::Arc => format!(
                    "%template(Boxed{}) ::rust::cxxbridge1::Box<::{}>;\n",
                    key.wrapper_name, key.wrapper_name
                ),
                RustWrapperType::Result => format!(
                    "%template(Boxed{}) ::rust::cxxbridge1::Box<::{}>;\n",
                    key.wrapper_name, key.wrapper_name
                ),
                RustWrapperType::Option => format!(
                    "%template(Boxed{}) ::rust::cxxbridge1::Box<::{}>;\n",
                    key.wrapper_name, key.wrapper_name
                ),
                RustWrapperType::Primitive => "".to_owned(),
            })
            .collect()
    }

    pub fn get_tokens(&self, module_name: &str) -> TokenStream {
        let module_name = Ident::new(module_name, Span::call_site());
        let module = &self.module;
        let generated = &self.generated;
        quote! {
            mod #module_name {
                use super::*;
                use std::fmt::Debug;
                use std::sync::{Arc, Mutex};
                #generated
                #module
            }
        }
        .into()
    }

    pub fn transform_module(mut module: ItemMod, for_cxx: bool) -> Result<Self, String> {
        let mut result: Self = Self::default();
        BindingModule::get_vec_of_extern_items_from_module(&mut module)?
            .iter_mut()
            .map(|extern_item| match extern_item {
                ForeignItem::Fn(function) => {
                    result.transformer.transform_function(function, for_cxx)
                }
                ForeignItem::Type(ForeignItemType { ident, .. }) => {
                    *ident = Transformer::create_wrapper_name("Rust", &ident.to_string()).ident;
                    Ok(())
                }
                _ => Ok(()),
            })
            .collect::<Result<(), String>>()?;
        BindingModule::get_vec_of_extern_items_from_module(&mut module)?.extend(vec![
            parse_quote!(
                type ResultFfiError;
            ),
        ]);
        result.module = Some(module);
        result.generate_rust_wrappers_in_extern_mod(for_cxx)?;
        result.generate_rust_wrappers_definitions(for_cxx);
        result.generate_custom_types_wrappers();
        result.generate_global_functions_wrappers();
        Ok(result)
    }

    /////////////////////////////////////
    /////////////////////////////////////
    /////////////////////////////////////

    pub fn get_cxx_module(&self) -> ItemMod {
        let mut module = self.module.as_ref().unwrap().clone();
        module.attrs = vec![];
        parse_quote! { #[cxx::bridge] #module }
    }

    pub fn get_swift_module(&self) -> ItemMod {
        let mut module = self.module.as_ref().unwrap().clone();
        module.attrs = vec![];
        parse_quote! { #[swift_bridge::bridge] #module }
    }

    pub fn parse_swift(input: TokenStream) -> Result<Self, String> {
        let module: ItemMod = parse_quote!( #[swift_bridge::bridge] #input );
        BindingModule::transform_module(module, false)
    }

    pub fn parse_cxx(input: TokenStream) -> Result<Self, String> {
        let module: ItemMod = parse_quote!( #[cxx::bridge] #input );
        BindingModule::transform_module(module, true)
    }
}
