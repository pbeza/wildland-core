use std::collections::{HashMap, HashSet};

use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{
    parse_quote, FnArg, ForeignItemFn, GenericArgument, Pat, PatIdent, PathArguments, PathSegment,
    ReturnType, Type, TypeReference, TypeTraitObject,
};

use crate::binding_types::*;

#[derive(Default)]
pub struct Transformer {
    pub rust_types_wrappers: HashSet<WrapperType>,
    pub structures_wrappers: HashMap<WrapperType, Vec<Function>>,
    pub global_functions: Vec<Function>,
}

impl Transformer {
    pub fn transform_function(&mut self, function: &mut ForeignItemFn) -> Result<(), String> {
        let mut arguments = vec![];
        let mut associated_structure = None;
        let mut return_type = None;
        function
            .sig
            .inputs
            .iter_mut()
            .map(|argument| match argument {
                FnArg::Typed(argument) => {
                    let new_wrapper_type = self
                        .tansform_rust_type_into_wrapper(argument.ty.as_mut())
                        .expect("At least one type should be present");
                    if let Pat::Ident(PatIdent { ident, .. }) = argument.pat.as_ref() {
                        arguments.push(Arg {
                            arg_name: ident.clone(),
                            typ: new_wrapper_type.clone(),
                        });
                        if ident.to_string() == "self" {
                            associated_structure = Some(new_wrapper_type);
                        }
                    }
                    Ok(())
                }
                _ => Err(format!(
                    "Only typed arguments are supported (no bare `self`): {}",
                    function.sig.ident
                )),
            })
            .collect::<Result<(), String>>()?;

        if let ReturnType::Type(_, typ) = &mut function.sig.output {
            let new_wrapper_type = self
                .tansform_rust_type_into_wrapper(typ.as_mut())
                .expect("At least one type should be present in return type");
            ///////////////////////////////////
            // CXX hack: Return Boxed value: //   TODO: add some abstraction on top of that
            ///////////////////////////////////
            if let RustWrapperType::Identic = &new_wrapper_type.typ {
            } else {
                let new_name = &new_wrapper_type.new_name;
                function.sig.output = parse_quote!( -> Box<#new_name>);
            }
            return_type = Some(new_wrapper_type);
        }
        if let Some(custom_type) = associated_structure {
            if !self.structures_wrappers.contains_key(&custom_type) {
                self.structures_wrappers.insert(custom_type.clone(), vec![]);
            }
            let fn_vector = self
                .structures_wrappers
                .get_mut(&custom_type)
                .expect("There must be a value");
            fn_vector.push(Function {
                parsed_items: function.clone(),
                arguments,
                return_type,
            })
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
                .map(|generic_argument| match generic_argument {
                    GenericArgument::Type(typ) => Some(typ),
                    _ => None,
                })
                .flatten(),
            _ => None,
        }
    }

    fn tansform_rust_type_into_wrapper(&mut self, typ: &mut Type) -> Option<WrapperType> {
        match typ {
            Type::Path(path) => {
                path.path
                    .segments
                    .first_mut()
                    .map(|path_segment| {
                        let new_wrapper_type = match path_segment.ident.to_string().as_str() {
                            "Result" => {
                                // TODO: Add void type in Result.
                                let inner_path = Transformer::get_inner_generic_type(path_segment);
                                if let Some(inner_path) = inner_path {
                                    if let Some(inner_type_name) =
                                        self.tansform_rust_type_into_wrapper(inner_path)
                                    {
                                        *path_segment = Transformer::create_wrapper_name(
                                            "Result",
                                            &inner_type_name.new_name.to_string(),
                                        );
                                        Some(WrapperType {
                                            name: inner_type_name.get_new_type(),
                                            new_name: path_segment.ident.clone(),
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
                                            &inner_type_name.new_name.to_string(),
                                        );
                                        Some(WrapperType {
                                            name: inner_type_name.get_new_type(),
                                            new_name: path_segment.ident.clone(),
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
                                        if let identical @ ("u8" | "u16" | "u32" | "u64" | "u128"
                                        | "i8" | "i16" | "i32" | "i64"
                                        | "i128" | "f8" | "f16" | "f32"
                                        | "f64" | "f128" | "String" | "usize") =
                                            inner_type_name.new_name.to_string().as_str()
                                        {
                                            let new_id = Ident::new(&identical, Span::call_site());
                                            Some(WrapperType {
                                                name: parse_quote!( #new_id ),
                                                new_name: new_id,
                                                typ: RustWrapperType::Identic,
                                                inner_type: Some(inner_type_name.into()),
                                            })
                                        } else {
                                            *path_segment = Transformer::create_wrapper_name(
                                                "Vec",
                                                &inner_type_name.new_name.to_string(),
                                            );
                                            Some(WrapperType {
                                                name: inner_type_name.get_new_type(),
                                                new_name: path_segment.ident.clone(),
                                                typ: RustWrapperType::Vector,
                                                inner_type: Some(inner_type_name.into()),
                                            })
                                        }
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
                                    if let Some(inner_type_name) =
                                        self.tansform_rust_type_into_wrapper(inner_path)
                                    {
                                        *path_segment = Transformer::create_wrapper_name(
                                            "Shared",
                                            &inner_type_name.new_name.to_string(),
                                        );
                                        Some(WrapperType {
                                            name: inner_type_name.get_new_type(),
                                            new_name: path_segment.ident.clone(),
                                            typ: RustWrapperType::Arc,
                                            inner_type: Some(inner_type_name.into()),
                                        })
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            }
                            "Mutex" => {
                                let inner_path = Transformer::get_inner_generic_type(path_segment);
                                if let Some(inner_path) = inner_path {
                                    if let Some(inner_type_name) =
                                        self.tansform_rust_type_into_wrapper(inner_path)
                                    {
                                        *path_segment = Transformer::create_wrapper_name(
                                            "Mutex",
                                            &inner_type_name.new_name.to_string(),
                                        );
                                        Some(WrapperType {
                                            name: inner_type_name.get_new_type(),
                                            new_name: path_segment.ident.clone(),
                                            typ: RustWrapperType::Mutex,
                                            inner_type: Some(inner_type_name.into()),
                                        })
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            }
                            identical @ ("u8" | "u16" | "u32" | "u64" | "u128" | "i8" | "i16"
                            | "i32" | "i64" | "i128" | "f8" | "f16" | "f32"
                            | "f64" | "f128" | "String" | "usize") => {
                                let new_id = Ident::new(&identical, Span::call_site());
                                Some(WrapperType {
                                    name: parse_quote!( #new_id ),
                                    new_name: new_id,
                                    typ: RustWrapperType::Identic,
                                    inner_type: None,
                                })
                            }
                            custom_type => {
                                *path_segment =
                                    Transformer::create_wrapper_name("Rust", &custom_type);
                                let new_id = Ident::new(custom_type, Span::call_site());
                                Some(WrapperType {
                                    name: parse_quote!( #new_id ),
                                    new_name: path_segment.ident.clone(),
                                    typ: RustWrapperType::Custom,
                                    inner_type: None,
                                })
                            }
                        };
                        if let Some(new_wrapper_type) = &new_wrapper_type {
                            self.rust_types_wrappers.insert(new_wrapper_type.clone());
                        }
                        new_wrapper_type
                    })
                    .flatten()
            }
            Type::Reference(TypeReference { elem, .. }) => {
                self.tansform_rust_type_into_wrapper(elem.as_mut())
            }
            Type::TraitObject(TypeTraitObject { bounds, .. }) => {
                // bounds.it;  Transformer::create_wrapper_name("Dyn", "Identity");
                // TODO:
                let new_wrapper_type = WrapperType {
                    name: parse_quote!(dyn Identity),
                    new_name: Ident::new("DynIdentity", Span::call_site()),
                    typ: RustWrapperType::DynTrait,
                    inner_type: None,
                };
                *typ = parse_quote!(DynIdentity);
                self.rust_types_wrappers.insert(new_wrapper_type.clone());
                Some(new_wrapper_type)
            }
            _ => None,
        }
    }

    pub fn create_wrapper_name(outer: &str, inner: &str) -> PathSegment {
        let name = Ident::new(&format!("{}{}", outer, inner), Span::call_site());
        let path_segment = quote! { #name };
        parse_quote!(#path_segment)
    }
}
