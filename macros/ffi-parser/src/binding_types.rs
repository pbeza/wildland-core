use proc_macro2::Ident;
use syn::{parse_quote, ForeignItemFn, Type};

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub enum RustWrapperType {
    Result { name: Type, new_name: Ident },
    Option { name: Type, new_name: Ident },
    Vector { name: Type, new_name: Ident },
    Custom { name: Type, new_name: Ident },
    Identic { name: Type, new_name: Ident },
}

impl RustWrapperType {
    pub fn get_new_name(&self) -> Ident {
        match self {
            Self::Result { new_name, .. } => new_name.clone(),
            Self::Option { new_name, .. } => new_name.clone(),
            Self::Vector { new_name, .. } => new_name.clone(),
            Self::Custom { new_name, .. } => new_name.clone(),
            Self::Identic { new_name, .. } => new_name.clone(),
        }
    }
    pub fn get_new_type(&self) -> Type {
        let id = self.get_new_name();
        parse_quote!( #id )
    }
}

#[derive(Debug)]
pub struct Arg {
    pub arg_name: Ident,
    pub typ: RustWrapperType,
}

#[derive(Debug)]
pub struct Function {
    pub parsed_items: ForeignItemFn,
    pub arguments: Vec<Arg>,
    pub return_type: Option<RustWrapperType>,
}
