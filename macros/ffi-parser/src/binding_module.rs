/// An implementation of a translator that can parse some specific Rust modules and adjust
/// them to the form understandable for `swift-bridge` crates.
///
use crate::{
    binding_types::*,
    cpp_ffi_interface::{generate_cpp_interface_file, GeneratedFilesContent},
    extern_module_translator::ExternModuleTranslator,
};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{parse_quote, Item, ItemFn, ItemForeignMod, ItemMod, Type};

/// The structure consist of:
/// - extern blocks translator module, i.e. `extern "Rust"`,
/// - translated Rust module,
/// - generated implementations blocks of the wrappers
/// - user provided `use` statements (useful for adding trait implementations).
///
/// # Example Rust module that can be translated:
/// ```rust
/// mod ffi {
///     use super::SomeTrait;
///     extern "Rust" {
///         type CustomType;
///         fn return_result_with_dynamic_type(self: &CustomType) -> Result<Arc<Mutex<dyn SomeTrait>>>;
///         fn return_another_custom_type(self: &CustomType) -> AnotherCustomType;
///
///         type AnotherCustomType;
///         fn take_primitive_type_and_return_primitive_type(self: &AnotherCustomType, a: u32) -> String;        
///         
///         fn some_trait_method(self: &Arc<Mutex<dyn SomeTrait>>);
///     }
/// }
///
pub struct BindingModule {
    extern_module_translator: ExternModuleTranslator,
    module: ItemMod,
    wrappers_impls: TokenStream,
    custom_uses: Vec<Item>,
}

impl BindingModule {
    /// The method tranlates Rust module containing `extern "Rust"` section into form usable
    /// for `swift-bridge`.
    ///
    pub fn translate_module(mut module: ItemMod) -> Result<Self, String> {
        let extern_module_translator = ExternModuleTranslator::translate_external_module_for_swift(
            BindingModule::get_extern_mod_from_module(&mut module)?,
        )?;
        let mut result = Self {
            extern_module_translator,
            module,
            wrappers_impls: quote!(),
            custom_uses: vec![],
        };
        result.module.attrs = vec![parse_quote!( #[swift_bridge::bridge] )];
        result.take_out_all_use_occurences()?;
        result.generate_wrappers_definitions();
        result.generate_impl_blocks_for_wrappers_with_methods();
        result.generate_wrappers_of_global_functions();
        Ok(result)
    }

    /// Returns only a translated Rust module. In particular this is useful for the build script in order
    /// to feed the module compilers of `swift-bridge-build` crates.
    ///
    pub fn get_module(&self) -> &ItemMod {
        &self.module
    }

    /// Takes a stream of tokens and translates it into a form usable for `swift-bridge`.
    ///
    pub fn parse(input: TokenStream) -> Result<Self, String> {
        let module: ItemMod = parse_quote!( #input );
        BindingModule::translate_module(module)
    }

    /// Method generates C++ and SWIG glue code needed to generate glue code
    /// for other taget languages like Java, C#, Python etc.
    ///
    pub fn generate_cpp_interface_file(&self) -> GeneratedFilesContent {
        generate_cpp_interface_file(&self.extern_module_translator)
    }

    /// Returns the whole generated code in the form of token stream.
    /// It wrapps the derived module into another one in order to avoid custom types
    /// and wrappers types names collisions.
    ///
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

    ///
    /// TODO: add doc here
    ///
    fn get_extern_mod_from_module(module: &mut ItemMod) -> Result<&mut ItemForeignMod, String> {
        module
            .content
            .as_mut()
            .ok_or_else(|| "The module is empty.".to_owned())?
            .1
            .iter_mut()
            .find_map(|module_item| match module_item {
                Item::ForeignMod(rust_module) => Some(rust_module),
                _ => None,
            })
            .ok_or_else(|| "Expected `extern \"Rust\"` within the module.".to_owned())
    }

    ///
    /// TODO: add doc here
    ///
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

    ///
    /// TODO: add doc here
    ///
    fn generate_wrappers_definitions(&mut self) {
        let tokens = self
            .extern_module_translator
            .rust_types_wrappers
            .iter()
            .flat_map(|wrapper| {
                let original_type_name = &wrapper.original_type_name;
                let return_original_type_name: Type = parse_quote! (#original_type_name);
                let error_type_name: Type = parse_quote!(ErrorType);

                generate_wrapper_definition(wrapper, return_original_type_name, error_type_name)
            });
        self.wrappers_impls.extend(tokens);
    }

    ///
    /// TODO: add doc here
    ///
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
                    .map(|Arg { arg_name, .. }| quote! {  unsafe { std::mem::transmute(#arg_name) } });
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
                                // .map(|ok| ok.into())
                                // .map_err(|err| err.into())).into()
                                .map(|ok| unsafe { std::mem::transmute(ok) })
                                .map_err(|err| unsafe { std::mem::transmute(err) })).into()
                        }},
                        RustWrapperType::Option => quote! {{
                            #wrapper_name(#struct_name #fn_name( #(#args),* )
                                .map(|ok| ok.into())).into()
                        }},
                        _ => quote! {{ unsafe { std::mem::transmute(#struct_name #fn_name( #(#args),* ))} }}
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

    ///
    /// TODO: add doc here
    ///
    fn generate_impl_blocks_for_wrappers_with_methods(&mut self) {
        for (custom_type, functions) in &self.extern_module_translator.structures_wrappers {
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

    ///
    /// TODO: add doc here
    ///
    fn generate_wrappers_of_global_functions(&mut self) {
        let generated_functions = BindingModule::generate_function_based_on_its_signature(
            &self.extern_module_translator.global_functions,
            false,
            None,
        );
        let generated_custom_wrapper_types = quote! { #(#generated_functions)* };
        self.wrappers_impls.extend(generated_custom_wrapper_types);
    }
}

///
/// TODO: add doc here
///
fn generate_wrapper_definition(
    wrapper: &WrapperType,
    return_original_type_name: Type,
    error_type_name: Type,
) -> TokenStream {
    let original_type_name = &wrapper.original_type_name;
    let wrapper_name = &wrapper.wrapper_name;
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
        RustWrapperType::Custom => {
            quote! {
                #[derive(Clone, Debug)]
                pub struct #wrapper_name(super::#original_type_name);
                impl From<super::#original_type_name> for #wrapper_name {
                    fn from(w: super::#original_type_name) -> #wrapper_name {
                        #wrapper_name(w)
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
    tokens
}
