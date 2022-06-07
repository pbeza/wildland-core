use proc_macro2::Ident;
use syn::{parse_quote, ForeignItemFn, Type};

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub enum RustWrapperType {
    Result,
    Option,
    Vector,
    Arc,
    Mutex,
    DynTrait,
    Custom,
    Identic,
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct WrapperType {
    pub name: Type,
    pub new_name: Ident,
    pub typ: RustWrapperType,
    pub inner_type: Option<Box<WrapperType>>,
}

impl WrapperType {
    pub fn get_new_type(&self) -> Type {
        let id = &self.new_name;
        parse_quote!( #id )
    }
}

#[derive(Debug)]
pub struct Arg {
    pub arg_name: Ident,
    pub typ: WrapperType,
}

#[derive(Debug)]
pub struct Function {
    pub parsed_items: ForeignItemFn,
    pub arguments: Vec<Arg>,
    pub return_type: Option<WrapperType>,
}
