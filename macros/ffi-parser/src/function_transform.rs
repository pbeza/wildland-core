use std::collections::{HashMap, HashSet};

use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use syn::{
    parse_quote, FnArg, ForeignItemFn, GenericArgument, Pat, PatIdent, PathArguments, PathSegment,
    ReturnType, Type, TypeReference,
};

use crate::binding_types::*;

#[derive(Default)]
pub struct Transformer {
    pub rust_types_wrappers: HashSet<WrapperType>,
    pub structures_wrappers: HashMap<WrapperType, Vec<Function>>,
    pub global_functions: Vec<Function>,
}

impl Transformer {
    pub fn register_custom_type(&mut self, original_type_name: Ident) -> WrapperType {
        let new_name = Transformer::create_wrapper_name("Rust", &original_type_name.to_string());
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

    pub fn transform_function(
        &mut self,
        function: &mut ForeignItemFn,
        boxed_result: bool,
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
                        .expect("At least one type should be present");
                    if let Pat::Ident(PatIdent { ident, .. }) = argument.pat.as_ref() {
                        arguments.push(Arg {
                            arg_name: ident.clone(),
                            typ: new_wrapper_type.clone(),
                        });
                        if *ident == "self" {
                            associated_structure = Some(new_wrapper_type);
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
            if boxed_result && new_wrapper_type.typ != RustWrapperType::Primitive {
                if let Some(boxed_inner) = &new_wrapper_type.inner_type {
                    if !(new_wrapper_type.typ == RustWrapperType::Vector
                        && boxed_inner.typ == RustWrapperType::Primitive)
                    {
                        let wrapper_name = &new_wrapper_type.wrapper_name;
                        function.sig.output = parse_quote!( -> Box<#wrapper_name>);
                    }
                } else {
                    let wrapper_name = &new_wrapper_type.wrapper_name;
                    function.sig.output = parse_quote!( -> Box<#wrapper_name>);
                }
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

    fn get_inner_generic_type(path_segment: &mut PathSegment) -> Option<&mut Type> {
        match &mut path_segment.arguments {
            PathArguments::AngleBracketed(args) => args
                .args
                .first_mut()
                .and_then(|generic_argument| match generic_argument {
                    GenericArgument::Type(typ) => Some(typ),
                    _ => None,
                }),
            _ => None,
        }
    }

    fn tansform_rust_type_into_wrapper(&mut self, typ: &mut Type) -> Option<WrapperType> {
        match typ {
            Type::Path(path) => path
                .path
                .segments
                .first_mut()
                .and_then(|path_segment| {
                    let new_wrapper_type = match path_segment.ident.to_string().as_str() {
                        "Result" => {
                            let inner_path = Transformer::get_inner_generic_type(path_segment);
                            if let Some(inner_path) = inner_path {
                                if let Some(inner_type_name) =
                                    self.tansform_rust_type_into_wrapper(inner_path)
                                {
                                    *path_segment = Transformer::create_wrapper_name(
                                        "Result",
                                        &inner_type_name.wrapper_name.to_string(),
                                    );
                                    Some(WrapperType {
                                        original_type_name: inner_type_name.get_new_type(),
                                        wrapper_name: path_segment.ident.clone(),
                                        typ: RustWrapperType::Result,
                                        inner_type: Some(inner_type_name.into()),
                                    })
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }
                        "Option" => {
                            let inner_path = Transformer::get_inner_generic_type(path_segment);
                            if let Some(inner_path) = inner_path {
                                if let Some(inner_type_name) =
                                    self.tansform_rust_type_into_wrapper(inner_path)
                                {
                                    *path_segment = Transformer::create_wrapper_name(
                                        "Optional",
                                        &inner_type_name.wrapper_name.to_string(),
                                    );
                                    Some(WrapperType {
                                        original_type_name: inner_type_name.get_new_type(),
                                        wrapper_name: path_segment.ident.clone(),
                                        typ: RustWrapperType::Option,
                                        inner_type: Some(inner_type_name.into()),
                                    })
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }
                        "Vec" => {
                            let inner_path = Transformer::get_inner_generic_type(path_segment);
                            if let Some(inner_path) = inner_path {
                                if let Some(inner_type_name) =
                                    self.tansform_rust_type_into_wrapper(inner_path)
                                {
                                    if inner_type_name.typ != RustWrapperType::Primitive {
                                        *path_segment = Transformer::create_wrapper_name(
                                            "Vec",
                                            &inner_type_name.wrapper_name.to_string(),
                                        );
                                    }
                                    Some(WrapperType {
                                        original_type_name: inner_type_name.get_new_type(),
                                        wrapper_name: path_segment.ident.clone(),
                                        typ: RustWrapperType::Vector,
                                        inner_type: Some(inner_type_name.into()),
                                    })
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }
                        "Arc" => {
                            let inner_path = Transformer::get_inner_generic_type(path_segment);
                            if let Some(inner_path) = inner_path {
                                let original_type = inner_path.clone();
                                *path_segment = Transformer::create_wrapper_name(
                                    "Shared",
                                    &inner_path
                                        .to_token_stream()
                                        .to_string()
                                        .replace("dyn", "")
                                        .replace('<', "")
                                        .replace('>', "")
                                        .replace(' ', ""),
                                );
                                Some(WrapperType {
                                    original_type_name: original_type,
                                    wrapper_name: path_segment.ident.clone(),
                                    typ: RustWrapperType::Arc,
                                    inner_type: None,
                                })
                            } else {
                                None
                            }
                        }
                        primitive @ ("u8" | "u16" | "u32" | "u64" | "u128" | "i8" | "i16"
                        | "i32" | "i64" | "i128" | "f8" | "f16" | "f32" | "f64"
                        | "f128" | "String" | "usize") => {
                            let new_id = Ident::new(primitive, Span::call_site());
                            Some(WrapperType {
                                original_type_name: parse_quote!( #new_id ),
                                wrapper_name: new_id,
                                typ: RustWrapperType::Primitive,
                                inner_type: None,
                            })
                        }
                        custom_type => {
                            *path_segment = Transformer::create_wrapper_name("Rust", custom_type);
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

    pub fn create_wrapper_name(outer: &str, inner: &str) -> PathSegment {
        let original_type_name = Ident::new(&format!("{}{}", outer, inner), Span::call_site());
        let path_segment = quote! { #original_type_name };
        parse_quote!(#path_segment)
    }
}
