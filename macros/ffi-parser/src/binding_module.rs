use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse_quote, Block, ForeignItem, ForeignItemType, Item, ItemFn, ItemForeignMod, ItemMod,
};

use crate::{binding_types::*, function_transform::Transformer};

#[derive(Default)]
pub struct BindingModule {
    transformer: Transformer,
    module: Option<ItemMod>,
    generated: TokenStream,
}

// TODO: Add generating wildland.i file.

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

    fn generate_rust_wrappers_in_extern_mod(&mut self) -> Result<(), String> {
        for wrapper in &self.transformer.rust_types_wrappers {
            let tokens = match wrapper {
                RustWrapperType::Result { name, new_name, .. } => quote! {
                    extern "Rust" {
                        type #new_name;
                        fn boxed_unwrap(self: &#new_name) -> Box<#name>;
                        fn boxed_unwrap_err(self: &#new_name) -> Box<ResultFfiError>;
                        fn is_ok(self: &#new_name) -> bool;
                    }
                },
                RustWrapperType::Option { name, new_name, .. } => quote! {
                    extern "Rust" {
                        type #new_name;
                        fn boxed_unwrap(self: &#new_name) -> Box<#name>;
                        fn is_some(self: &#new_name) -> bool;
                    }
                },
                RustWrapperType::Vector { name, new_name, .. } => quote! {
                    extern "Rust" {
                        type #new_name;
                        fn at(self: &#new_name) -> Box<#name>;
                        fn size(self: &#new_name) -> usize;
                    }
                },
                RustWrapperType::Custom { new_name, .. } => quote! {
                    extern "Rust" {
                        type #new_name;
                    }
                },
                // Those types will be created along with
                // the custom Rust types wrappers:
                _ => continue,
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

    fn generate_rust_wrappers_definitions(&mut self) {
        for wrapper in &self.transformer.rust_types_wrappers {
            let tokens: TokenStream = match wrapper {
                RustWrapperType::Result { name, new_name } => {
                    quote! { type #new_name = Res<#name>; }.into()
                }
                RustWrapperType::Option { name, new_name } => {
                    quote! { type #new_name = Opt<#name>; }.into()
                }
                RustWrapperType::Vector { name, new_name } => {
                    quote! { type #new_name = Array<#name>; }.into()
                }
                // Those types will be created along with
                // the custom Rust types wrappers:
                _ => quote! {}.into(),
            };
            self.generated.extend(tokens);
        }
    }

    fn generate_function_body(
        functions: &Vec<Function>,
        struct_name: TokenStream,
        skip_first: bool,
    ) -> Vec<ItemFn> {
        functions
            .iter()
            .map(|function| {
                let fn_name = &function.parsed_items.sig.ident;
                let args = function
                    .arguments
                    .iter()
                    .skip(skip_first as usize)
                    .map(|Arg { arg_name, typ }| match typ {
                        RustWrapperType::Result { .. } => {
                            quote! { #arg_name.0 }
                        }
                        RustWrapperType::Option { .. } => {
                            quote! { #arg_name.0 }
                        }
                        RustWrapperType::Vector { .. } => {
                            quote! { #arg_name.0 }
                        }
                        RustWrapperType::Custom { .. } => {
                            quote! { #arg_name.into() }
                        }
                        RustWrapperType::Identic { .. } => {
                            quote! { #arg_name }
                        }
                    })
                    .collect::<Vec<_>>();
                let fn_call = match &function.return_type {
                    Some(RustWrapperType::Result { name, new_name, .. }) => {
                        quote! {
                            {
                                #new_name::from(
                                    #struct_name #fn_name( #(#args),* ).map(|ok| #name::from(ok))
                                ).into()
                            }
                        }
                    }
                    Some(RustWrapperType::Option { name, new_name, .. }) => {
                        quote! {
                            {
                                #new_name::from(
                                    #struct_name #fn_name( #(#args),* ).map(|ok| #name::from(ok))
                                ).into()
                            }
                        }
                    }
                    Some(RustWrapperType::Vector { new_name, .. }) => {
                        quote! {{ #new_name(#struct_name #fn_name( #(#args),* )).into() }}
                    }
                    Some(RustWrapperType::Custom { new_name, .. }) => {
                        quote! {{ #new_name::from(#struct_name #fn_name( #(#args),* )).into() }}
                    }
                    Some(RustWrapperType::Identic { .. }) => {
                        quote! {{ #struct_name #fn_name( #(#args),* ) }}
                    }
                    None => quote! {{ #struct_name #fn_name( #(#args),* ); }},
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
            if let RustWrapperType::Custom { name, new_name } = custom_type {
                let generated_functions = BindingModule::generate_function_body(
                    &functions,
                    quote!( self.0.lock().unwrap(). ),
                    true,
                );
                let generated_custom_wrapper_types: TokenStream = quote! {
                    #[derive(Clone, Debug)]
                    struct #new_name(Arc<Mutex<#name>>);
                    impl #new_name {
                        #(#generated_functions)*
                    }
                    impl From<#name> for #new_name {
                        fn from(w: #name) -> #new_name {
                            #new_name(Arc::new(Mutex::new(w)))
                        }
                    }
                }
                .into();
                self.generated.extend(generated_custom_wrapper_types);
            }
        }
    }

    fn generate_global_functions_wrappers(&mut self) {
        let generated_functions = BindingModule::generate_function_body(
            &self.transformer.global_functions,
            quote!(crate::),
            false,
        );

        let generated_custom_wrapper_types: TokenStream = quote! {
                #(#generated_functions)*
        }
        .into();
        self.generated.extend(generated_custom_wrapper_types);
    }

    pub fn get_cxx_module(&self) -> ItemMod {
        let mut module = self.module.as_ref().unwrap().clone();
        module.attrs = vec![];
        parse_quote! {
            #[cxx::bridge]
            #module
        }
    }

    pub fn get_tokens(&self) -> TokenStream {
        let module = &self.module;
        let generated = &self.generated;
        let result_rs = syn::parse_file(include_str!("included/result.rs")).unwrap();
        let option_rs = syn::parse_file(include_str!("included/option.rs")).unwrap();
        let array_rs = syn::parse_file(include_str!("included/array.rs")).unwrap();
        quote! {
            use std::sync::{Arc, Mutex};
            #generated
            #module
            #result_rs
            #option_rs
            #array_rs
        }
        .into()
    }

    pub fn parse(input: TokenStream) -> Result<Self, String> {
        let module_module: ItemMod = parse_quote!(
             #[cxx::bridge]    // TODO: remove namespace in both places.
             #input
        );
        BindingModule::transform_module(module_module)
    }

    pub fn transform_module(mut module: ItemMod) -> Result<Self, String> {
        let mut result: Self = Self::default();
        BindingModule::get_vec_of_extern_items_from_module(&mut module)?
            .iter_mut()
            .map(|extern_item| match extern_item {
                ForeignItem::Fn(function) => result.transformer.transform_function(function),
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
        result.generate_rust_wrappers_in_extern_mod()?;
        result.generate_rust_wrappers_definitions();
        result.generate_custom_types_wrappers();
        result.generate_global_functions_wrappers();
        Ok(result)
    }
}
