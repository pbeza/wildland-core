use crate::{binding_types::*, function_transform::ModuleTranslator};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{parse_quote, ForeignItem, ForeignItemType, Item, ItemFn, ItemForeignMod, ItemMod, Type};

pub struct BindingModule {
    module_translator: ModuleTranslator,
    module: ItemMod,
    wrappers_impls: TokenStream,
    custom_uses: Vec<Item>,
}

impl BindingModule {
    pub fn translate_module(module: ItemMod, for_cxx: bool) -> Result<Self, String> {
        let mut result = Self {
            module_translator: ModuleTranslator::default(),
            module,
            wrappers_impls: quote!(),
            custom_uses: vec![],
        };
        if for_cxx {
            result.module.attrs = vec![parse_quote!( #[cxx::bridge] )];
        } else {
            result.module.attrs = vec![parse_quote!( #[swift_bridge::bridge] )];
        }
        result.take_out_all_use_occurences()?;
        result.replace_every_type_in_each_item_with_wrappers_in_module(for_cxx)?;
        result.generate_wrappers_types_and_methods_within_extern_module(for_cxx)?;
        result.generate_wrappers_definitions(for_cxx);
        result.generate_impl_blocks_for_wrappers_with_methods();
        result.generate_wrappers_of_global_functions();
        Ok(result)
    }

    pub fn get_module(&self) -> &ItemMod {
        &self.module
    }

    pub fn parse_swift(input: TokenStream) -> Result<Self, String> {
        let module: ItemMod = parse_quote!( #input );
        BindingModule::translate_module(module, false)
    }

    pub fn parse_cxx(input: TokenStream) -> Result<Self, String> {
        let module: ItemMod = parse_quote!( #input );
        BindingModule::translate_module(module, true)
    }

    pub fn generate_swig_interface_file_from_cxx_module(&self) -> String {
        self.module_translator
            .rust_types_wrappers
            .iter()
            .map(|key| match key.typ {
                RustWrapperType::Custom
                | RustWrapperType::Arc
                | RustWrapperType::Result
                | RustWrapperType::Option => format!(
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
                RustWrapperType::VectorPrimitive => format!(
                    "%template(Vec{}) ::rust::cxxbridge1::Vec<::rust::cxxbridge1::{}>;\n",
                    key.inner_type
                        .as_ref()
                        .expect("Vector has to have inner generic type")
                        .wrapper_name,
                    key.inner_type
                        .as_ref()
                        .expect("Vector has to have inner generic type")
                        .wrapper_name
                ),
                RustWrapperType::Primitive => "".to_owned(),
            })
            .collect()
    }

    pub fn get_tokens(&self, module_name: &str) -> TokenStream {
        let module_name = Ident::new(module_name, Span::call_site());
        let module = &self.module;
        let wrappers_impls = &self.wrappers_impls;
        let custom_uses = &self.custom_uses;
        quote! {
            mod #module_name {
                #(#custom_uses);*
                use std::fmt::Debug;
                use std::sync::{Arc, Mutex};
                #wrappers_impls
                #module
            }
        }
    }

    /////////////////////////////////////////////////////////
    /////////////////////////////////////////////////////////
    ///////////////////////////////////////////////////////// Private implementation:
    /////////////////////////////////////////////////////////

    fn get_vec_of_extern_items_from_module(
        module: &mut ItemMod,
    ) -> Result<&mut Vec<ForeignItem>, String> {
        module
            .content
            .as_mut()
            .ok_or_else(|| "The module is empty.".to_owned())?
            .1
            .iter_mut()
            .find_map(|module_item| match module_item {
                Item::ForeignMod(rust_module) => Some(&mut rust_module.items),
                _ => None,
            })
            .ok_or_else(|| "Expected `extern \"Rust\"` within the module.".to_owned())
    }

    fn take_out_all_use_occurences(&mut self) -> Result<(), String> {
        let mod_items_vector = &mut self
            .module
            .content
            .as_mut()
            .ok_or_else(|| "The module is empty.".to_owned())?
            .1;
        self.custom_uses = mod_items_vector
            .iter()
            .filter(|module_item| matches!(module_item, Item::Use(_)))
            .cloned()
            .collect();
        mod_items_vector.retain(|module_item| !matches!(module_item, Item::Use(_)));
        Ok(())
    }

    fn generate_wrappers_types_and_methods_within_extern_module(
        &mut self,
        boxed_result: bool,
    ) -> Result<(), String> {
        for wrapper in &self.module_translator.rust_types_wrappers {
            let wrapper_name = &wrapper.wrapper_name;
            let original_type_name = &wrapper.original_type_name;
            let return_original_type_name: Type = if boxed_result {
                parse_quote! ( Box<#original_type_name> )
            } else {
                parse_quote! (#original_type_name)
            };
            let error_type_name: Type = if boxed_result {
                parse_quote!(Box<ErrorType>)
            } else {
                parse_quote!(ErrorType)
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
                RustWrapperType::Vector => quote! {
                    extern "Rust" {
                        type #wrapper_name;
                        fn at(self: &#wrapper_name) -> #return_original_type_name;
                        fn size(self: &#wrapper_name) -> usize;
                    }
                },
                RustWrapperType::Arc => quote! {
                    extern "Rust" { type #wrapper_name; }
                },
                _ => quote! { extern "Rust" {} },
            };
            let generated_module_items: ItemForeignMod = parse_quote!(#tokens);
            BindingModule::get_vec_of_extern_items_from_module(&mut self.module)?
                .extend(generated_module_items.items);
        }
        Ok(())
    }

    fn generate_wrappers_definitions(&mut self, boxed_result: bool) {
        for wrapper in &self.module_translator.rust_types_wrappers {
            let wrapper_name = &wrapper.wrapper_name;
            let original_type_name = &wrapper.original_type_name;
            let return_original_type_name: Type = if boxed_result {
                parse_quote! ( Box<#original_type_name> )
            } else {
                parse_quote! (#original_type_name)
            };
            let error_type_name: Type = if boxed_result {
                parse_quote!(Box<ErrorType>)
            } else {
                parse_quote!(ErrorType)
            };
            let tokens: TokenStream = match &wrapper.typ {
                RustWrapperType::Result => {
                    quote! {
                        pub struct #wrapper_name(Result<#original_type_name, ErrorType>);
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
                },
                RustWrapperType::Vector => quote! {
                    pub struct #wrapper_name(Vec<#original_type_name>);
                    impl #wrapper_name {
                        pub fn at(&self, elem: usize) -> #return_original_type_name {
                            self.0[elem].clone()
                        }
                        pub fn size(&self) -> usize {
                            self.0.len()
                        }
                    }
                },
                RustWrapperType::Custom => {
                    quote! {
                        #[derive(Clone, Debug)]
                        pub struct #wrapper_name(super::#original_type_name);
                        impl From<super::#original_type_name> for #wrapper_name {
                            fn from(w: super::#original_type_name) -> #wrapper_name {
                                #wrapper_name(w)
                            }
                        }
                        impl<'a> From<&'a Box<#wrapper_name>> for &'a super::#original_type_name {
                            fn from(w: &'a Box<#wrapper_name>) -> &'a super::#original_type_name {
                                &w.as_ref().0
                            }
                        }
                        impl<'a> From<&'a #wrapper_name> for &'a super::#original_type_name {
                            fn from(w: &'a #wrapper_name) -> &'a super::#original_type_name {
                                &w.0
                            }
                        }
                    }
                }
                RustWrapperType::Arc => {
                    quote! {
                        #[derive(Clone, Debug)]
                        pub struct #wrapper_name(Arc<#original_type_name>);
                        impl From<Arc<#original_type_name>> for #wrapper_name {
                            fn from(w: Arc<#original_type_name>) -> #wrapper_name {
                                #wrapper_name(w)
                            }
                        }
                    }
                }
                _ => quote! {},
            };
            self.wrappers_impls.extend(tokens);
        }
    }

    fn generate_function_based_on_its_signature(
        functions: &[Function],
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
                    .map(|Arg { arg_name, .. }| quote! {  #arg_name.into() });
                let struct_name: TokenStream = if skip_first {
                    if let Some(custom_self) = &custom_self {
                        custom_self.clone()
                    } else {
                        quote! { self.0. }
                    }
                } else {
                    quote! { crate:: }
                };
                let function_body = if let Some(wrapper) = &function.return_type {
                    let wrapper_name = &wrapper.wrapper_name;
                    match wrapper.typ {
                        RustWrapperType::Result => quote! {{
                            #wrapper_name(#struct_name #fn_name( #(#args),* )
                                .map(|ok| ok.into())
                                .map_err(|err| err.into())).into()
                        }},
                        RustWrapperType::Option => quote! {{
                            #wrapper_name(#struct_name #fn_name( #(#args),* )
                                .map(|ok| ok.into())).into()
                        }},
                        _ => quote! {{ #wrapper_name::from(#struct_name #fn_name( #(#args),* )).into() }}
                    }
                } else {
                    quote! {{ #struct_name #fn_name( #(#args),* ); }}
                };
                ItemFn {
                    attrs: function.parsed_items.attrs.clone(),
                    vis: function.parsed_items.vis.clone(),
                    sig: function.parsed_items.sig.clone(),
                    block: parse_quote!(#function_body),
                }
            })
            .collect()
    }

    fn generate_impl_blocks_for_wrappers_with_methods(&mut self) {
        for (custom_type, functions) in &self.module_translator.structures_wrappers {
            let wrapper_name = &custom_type.wrapper_name;
            match custom_type.typ {
                RustWrapperType::Custom => {
                    let generated_functions =
                        BindingModule::generate_function_based_on_its_signature(
                            functions, true, None,
                        );
                    let generated_wrapper_impl: TokenStream = quote! {
                        impl #wrapper_name {
                            #(#generated_functions)*
                        }
                    };
                    self.wrappers_impls.extend(generated_wrapper_impl);
                }
                RustWrapperType::Arc => {
                    let generated_functions =
                        BindingModule::generate_function_based_on_its_signature(
                            functions,
                            true,
                            // This is used to upack T from Arc<Mutex<T>>:
                            // Note: Only Arc<Mutex<>> is supported,
                            //       Arc<T> without Mutex will cause an error.
                            Some(quote! { self.0.lock().unwrap(). }),
                        );
                    let generated_wrapper_impl: TokenStream = quote! {
                        impl #wrapper_name {
                            #(#generated_functions)*
                        }
                    };
                    self.wrappers_impls.extend(generated_wrapper_impl);
                }
                _ => {}
            }
        }
    }

    fn generate_wrappers_of_global_functions(&mut self) {
        let generated_functions = BindingModule::generate_function_based_on_its_signature(
            &self.module_translator.global_functions,
            false,
            None,
        );
        let generated_custom_wrapper_types = quote! { #(#generated_functions)* };
        self.wrappers_impls.extend(generated_custom_wrapper_types);
    }

    fn replace_every_type_in_each_item_with_wrappers_in_module(
        &mut self,
        for_cxx: bool,
    ) -> Result<(), String> {
        BindingModule::get_vec_of_extern_items_from_module(&mut self.module)?
            .iter_mut()
            .try_for_each(|extern_item| match extern_item {
                ForeignItem::Fn(function) => self
                    .module_translator
                    .replace_arg_types_with_wrappers(function, for_cxx),
                ForeignItem::Type(ForeignItemType { ident, .. }) => {
                    let wrapper_type = self.module_translator.register_custom_type(ident.clone());
                    *ident = wrapper_type.wrapper_name;
                    Ok(())
                }
                _ => Ok(()),
            })
    }
}
