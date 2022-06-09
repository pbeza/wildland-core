use crate::binding_types::*;
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use std::collections::{HashMap, HashSet};
use syn::{
    parse_quote, FnArg, ForeignItem, ForeignItemFn, ForeignItemType, GenericArgument,
    ItemForeignMod, Pat, PatIdent, PathArguments, PathSegment, ReturnType, Type, TypeReference,
};

///
/// TODO: add doc here
///
#[derive(Default)]
pub struct ExternModuleTranslator {
    pub rust_types_wrappers: HashSet<WrapperType>,
    pub structures_wrappers: HashMap<WrapperType, Vec<Function>>,
    pub global_functions: Vec<Function>,
}

impl ExternModuleTranslator {
    ///
    /// TODO: add doc here
    ///
    pub fn translate_external_module_for_swift(
        extern_module: &mut ItemForeignMod,
    ) -> Result<Self, String> {
        let mut result = ExternModuleTranslator::default();
        result.replace_every_type_in_each_item_with_wrappers_in_module(extern_module, false)?;
        result.add_wrappers_types_and_methods_to_extern_module(extern_module)?;
        Ok(result)
    }

    ///
    /// TODO: add doc here
    ///
    pub fn translate_external_module_for_cxx(
        extern_module: &mut ItemForeignMod,
    ) -> Result<Self, String> {
        let mut result = ExternModuleTranslator::default();
        result.replace_every_type_in_each_item_with_wrappers_in_module(extern_module, true)?;
        result.add_boxed_wrappers_types_and_methods_to_extern_module(extern_module)?;
        Ok(result)
    }

    ///
    /// TODO: add doc here
    ///
    fn add_wrappers_types_and_methods_to_extern_module(
        &mut self,
        extern_module: &mut ItemForeignMod,
    ) -> Result<(), String> {
        extern_module
            .items
            .extend(self.rust_types_wrappers.iter().flat_map(|wrapper| {
                let original_type_name = &wrapper.original_type_name;
                let return_original_type_name: Type = parse_quote! (#original_type_name);
                let error_type_name: Type = parse_quote!(ErrorType);
                generate_module_items(wrapper, return_original_type_name, error_type_name).items
            }));

        Ok(())
    }

    ///
    /// TODO: add doc here
    ///
    fn add_boxed_wrappers_types_and_methods_to_extern_module(
        &mut self,
        extern_module: &mut ItemForeignMod,
    ) -> Result<(), String> {
        extern_module
            .items
            .extend(self.rust_types_wrappers.iter().flat_map(|wrapper| {
                let original_type_name = &wrapper.original_type_name;
                let return_original_type_name: Type = parse_quote! ( Box<#original_type_name> );
                let error_type_name: Type = parse_quote!(Box<ErrorType>);
                generate_module_items(wrapper, return_original_type_name, error_type_name).items
            }));

        Ok(())
    }

    ///
    /// TODO: add doc here
    ///
    fn replace_every_type_in_each_item_with_wrappers_in_module(
        &mut self,
        extern_module: &mut ItemForeignMod,
        boxed_wrappers: bool,
    ) -> Result<(), String> {
        extern_module
            .items
            .iter_mut()
            .try_for_each(|extern_item| match extern_item {
                ForeignItem::Fn(function) => {
                    self.replace_arg_types_with_wrappers(function, boxed_wrappers)
                }
                ForeignItem::Type(ForeignItemType { ident, .. }) => {
                    let wrapper_type = self.register_custom_type(ident.clone());
                    *ident = wrapper_type.wrapper_name;
                    Ok(())
                }
                _ => Ok(()),
            })
    }

    /// Method stores a wrapper to some custom user's type.
    /// Can be used while parsing `type SomeUserType;` in
    /// order to register a new type `RustUserType` in
    /// the collection of intermediate-form wrappers.
    fn register_custom_type(&mut self, original_type_name: Ident) -> WrapperType {
        let new_name =
            ExternModuleTranslator::create_wrapper_name("", &original_type_name.to_string());
        let wrapper_name = Ident::new(&new_name.ident.to_string(), Span::call_site());
        let new_wrapper_type = WrapperType {
            original_type_name: parse_quote!( #original_type_name ),
            wrapper_name,
            typ: RustWrapperType::Custom,
            inner_type: None,
        };
        self.rust_types_wrappers.insert(new_wrapper_type.clone());
        new_wrapper_type
    }

    /// Method takes a mutable reference to the parsed foreign function declaration
    /// and replaces the arguments' types with wrapper types in place.
    ///
    /// Side effects: the method fills `rust_types_wrappers`, `structures_wrappers`
    ///               and `global_functions`.
    fn replace_arg_types_with_wrappers(
        &mut self,
        function: &mut ForeignItemFn,
        boxed_wrappers: bool,
    ) -> Result<(), String> {
        let mut arguments = vec![];
        let mut associated_structure = None;
        let mut return_type = None;
        function
            .sig
            .inputs
            .iter_mut()
            .try_for_each(|argument| match argument {
                FnArg::Typed(argument) => {
                    let new_wrapper_type = self
                        .tansform_rust_type_into_wrapper(argument.ty.as_mut())
                        .ok_or_else(|| {
                            format!(
                                "Unsupported type in function signature: {}",
                                function.sig.ident
                            )
                        })?;
                    if let Pat::Ident(PatIdent { ident, .. }) = argument.pat.as_ref() {
                        arguments.push(Arg {
                            arg_name: ident.clone(),
                            typ: new_wrapper_type.clone(),
                        });
                        if *ident == "self" {
                            associated_structure = Some(new_wrapper_type);
                        } else if boxed_wrappers
                            && new_wrapper_type.typ != RustWrapperType::Primitive
                            && new_wrapper_type.typ != RustWrapperType::VectorPrimitive
                        {
                            let wrapper_name = &new_wrapper_type.wrapper_name;
                            match &mut argument.ty.as_mut() {
                                Type::Reference(typ) => *typ = parse_quote!( &Box<#wrapper_name>),
                                Type::Path(typ) => *typ = parse_quote!( Box<#wrapper_name>),
                                _ => {}
                            }
                            argument.ty = parse_quote!( &Box<#wrapper_name>);
                        }
                    }
                    Ok(())
                }
                _ => Err(format!(
                    "Only typed arguments are supported (no bare `self`): {}",
                    function.sig.ident
                )),
            })?;

        if let ReturnType::Type(_, typ) = &mut function.sig.output {
            let new_wrapper_type = self
                .tansform_rust_type_into_wrapper(typ.as_mut())
                .ok_or("At least one type should be present in return type")?;
            if boxed_wrappers
                && new_wrapper_type.typ != RustWrapperType::Primitive
                && new_wrapper_type.typ != RustWrapperType::VectorPrimitive
            {
                let wrapper_name = &new_wrapper_type.wrapper_name;
                function.sig.output = parse_quote!( -> Box<#wrapper_name>);
            }
            return_type = Some(new_wrapper_type);
        }

        if let Some(custom_type) = associated_structure {
            self.structures_wrappers
                .entry(custom_type)
                .or_insert(vec![])
                .push(Function {
                    parsed_items: function.clone(),
                    arguments,
                    return_type,
                });
        } else {
            self.global_functions.push(Function {
                parsed_items: function.clone(),
                arguments,
                return_type,
            })
        }
        Ok(())
    }

    /// Translates <T> into T.
    /// Example:
    ///  * <Result<u8>> --> Result<u8>
    fn get_inner_generic_type(path_segment: &mut PathSegment) -> Option<&mut Type> {
        match &mut path_segment.arguments {
            PathArguments::AngleBracketed(args) => {
                args.args
                    .first_mut()
                    .and_then(|generic_argument| match generic_argument {
                        GenericArgument::Type(typ) => Some(typ),
                        _ => None,
                    })
            }
            _ => None,
        }
    }

    /// Method takes a type (for instance `Arc<Mutex<dyn SomeCustomType>>`)
    /// and creates one or more intermediate-form wrappers.
    /// Examples:
    /// The following wrappers are created out of Result<Arc<Mutex<dyn SomeCustomType>>>:
    ///  * ResultSharedMutexSomeCustomType
    ///  * SharedMutexSomeCustomType
    ///
    /// Note1: This method is called recursively for Results, Options and Vectors,
    ///       but Arc<...> and other types are treated as Opaque type.
    ///
    /// Note2: References are kept untouched.
    ///
    fn tansform_rust_type_into_wrapper(&mut self, typ: &mut Type) -> Option<WrapperType> {
        match typ {
            Type::Path(path) => path.path.segments.first_mut().and_then(|path_segment| {
                let new_wrapper_type = match path_segment.ident.to_string().as_str() {
                    "Result" => ExternModuleTranslator::get_inner_generic_type(path_segment)
                        .and_then(|inner_path| self.tansform_rust_type_into_wrapper(inner_path))
                        .map(|inner_type_name| {
                            *path_segment = ExternModuleTranslator::create_wrapper_name(
                                "Result",
                                &inner_type_name.wrapper_name.to_string(),
                            );
                            WrapperType {
                                original_type_name: inner_type_name.get_new_type(),
                                wrapper_name: path_segment.ident.clone(),
                                typ: RustWrapperType::Result,
                                inner_type: Some(inner_type_name.into()),
                            }
                        }),
                    "Option" => ExternModuleTranslator::get_inner_generic_type(path_segment)
                        .and_then(|inner_path| self.tansform_rust_type_into_wrapper(inner_path))
                        .map(|inner_type_name| {
                            *path_segment = ExternModuleTranslator::create_wrapper_name(
                                "Optional",
                                &inner_type_name.wrapper_name.to_string(),
                            );
                            WrapperType {
                                original_type_name: inner_type_name.get_new_type(),
                                wrapper_name: path_segment.ident.clone(),
                                typ: RustWrapperType::Option,
                                inner_type: Some(inner_type_name.into()),
                            }
                        }),
                    "Vec" => ExternModuleTranslator::get_inner_generic_type(path_segment)
                        .and_then(|inner_path| self.tansform_rust_type_into_wrapper(inner_path))
                        .map(|inner_type_name| {
                            if inner_type_name.typ != RustWrapperType::Primitive {
                                *path_segment = ExternModuleTranslator::create_wrapper_name(
                                    "Vec",
                                    &inner_type_name.wrapper_name.to_string(),
                                );
                            }
                            WrapperType {
                                original_type_name: inner_type_name.get_new_type(),
                                wrapper_name: path_segment.ident.clone(),
                                typ: if inner_type_name.typ == RustWrapperType::Primitive {
                                    RustWrapperType::VectorPrimitive
                                } else {
                                    RustWrapperType::Vector
                                },
                                inner_type: Some(inner_type_name.into()),
                            }
                        }),
                    "Arc" => ExternModuleTranslator::get_inner_generic_type(path_segment)
                        .cloned()
                        .map(|inner_path| {
                            let generated_inner_type_name = inner_path
                                .to_token_stream()
                                .to_string()
                                .replace("dyn", "")
                                .replace('<', "")
                                .replace('>', "")
                                .replace(' ', "");
                            *path_segment = ExternModuleTranslator::create_wrapper_name(
                                "Shared",
                                &generated_inner_type_name,
                            );
                            WrapperType {
                                original_type_name: inner_path,
                                wrapper_name: path_segment.ident.clone(),
                                typ: RustWrapperType::Arc,
                                inner_type: None,
                            }
                        }),
                    primitive @ ("u8" | "u16" | "u32" | "u64" | "u128" | "i8" | "i16" | "i32"
                    | "i64" | "i128" | "f8" | "f16" | "f32" | "f64" | "f128"
                    | "String" | "usize" | "bool") => {
                        let new_id = Ident::new(primitive, Span::call_site());
                        Some(WrapperType {
                            original_type_name: parse_quote!( #new_id ),
                            wrapper_name: new_id,
                            typ: RustWrapperType::Primitive,
                            inner_type: None,
                        })
                    }
                    custom_type => {
                        *path_segment =
                            ExternModuleTranslator::create_wrapper_name("", custom_type);
                        let new_id = Ident::new(custom_type, Span::call_site());
                        Some(WrapperType {
                            original_type_name: parse_quote!( #new_id ),
                            wrapper_name: path_segment.ident.clone(),
                            typ: RustWrapperType::Custom,
                            inner_type: None,
                        })
                    }
                };
                if let Some(new_wrapper_type) = &new_wrapper_type {
                    self.rust_types_wrappers.insert(new_wrapper_type.clone());
                }
                new_wrapper_type
            }),
            Type::Reference(TypeReference { elem, .. }) => {
                self.tansform_rust_type_into_wrapper(elem.as_mut())
            }
            _ => None,
        }
    }

    /// Takes two strings and transforms them into `PathSegment`.
    /// Example:
    ///  * create_wrapper_name("Rust", "SomeCustomStruct")  --> `RustSomeCustomStruct`.
    ///
    /// This return type is chosen for convenience.
    ///
    pub fn create_wrapper_name(outer: &str, inner: &str) -> PathSegment {
        let original_type_name = Ident::new(&format!("{}{}", outer, inner), Span::call_site());
        let path_segment = quote! { #original_type_name };
        parse_quote!(#path_segment)
    }
}

fn generate_module_items(
    wrapper: &WrapperType,
    return_original_type_name: Type,
    error_type_name: Type,
) -> ItemForeignMod {
    let wrapper_name = &wrapper.wrapper_name;
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
    parse_quote!(#tokens)
}
