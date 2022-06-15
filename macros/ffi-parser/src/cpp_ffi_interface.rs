use crate::{
    binding_types::{Function, RustWrapperType, WrapperType},
    extern_module_translator::ExternModuleTranslator,
};
use std::collections::HashSet;

pub struct GeneratedFilesContent {
    pub cpp_header: String,
    pub swig_interface: String,
}

const PREDEFINED: &str = "
#include <utility>
#include <cstring>

// TODO: add all the other primitive types here:
typedef unsigned int u32;
typedef unsigned char u8;

template <typename T>
class RustVec {
    void* self = nullptr;
    void (*drop)(void*);
    void (*__push)(void*, T*);
    void* (*__at)(void*, uintptr_t);
    size_t (*__size)(void*);
public:
    RustVec() = delete;
    RustVec(void* self,
            void (*drop)(void*),
            void (*__push)(void*, T*),
            void* (*__at)(void*, uintptr_t),
            size_t (*__size)(void*)) {
        this->self = self;
        this->drop = drop;
        this->__push = __push;
        this->__at = __at;
        this->__size = __size;
    }
    RustVec(RustVec&& a) : self(std::move(a.self)) { a.self = nullptr; };
    ~RustVec() {
        if(this->drop && this->self) {
            this->drop(this->self);
        }
    }
    void* get_ptr() { return this->self; }
    void push(T&& item) {
        this->__push(this->self, &item);    // TODO: Make sure that this actually works
    }
    T at(uintptr_t index) {
        return T(this->__at(this->self, index), false);
    }
    size_t size() {
        return (size_t) this->__size(this->self);
    }
};

class String {
    void* self = nullptr;
    bool is_owned = false;
public:
    String() = delete;
    String(void* self, bool is_owned) : self(self), is_owned(is_owned) {{ }}
    String(const char* str) {
        RustStr s = RustStr {
            str,
            strlen(str)
        };
        this->self = __swift_bridge__$RustString$new_with_str(s);
        this->is_owned = false;
    }
    String(String&& a)
        : self(a.self),
          is_owned(a.is_owned) { 
              a.self = nullptr; 
              a.is_owned = false;
    }
    ~String() {
        if(this->self && is_owned) {
            __swift_bridge__$RustString$_free(this->self);
        }
    }
    char* c_str() {
        RustStr s = __swift_bridge__$RustString$as_str(this->self);
        char* new_string = new char[s.len+1];
        strcpy(new_string, s.start);
        return new_string;
    }
    void* get_ptr() { return this->self; }
};
";

macro_rules! option_class {
    ($name:ident, $inner_type:ident) => {{
        format!(
            "
class {0} {{
    void* self = nullptr;
    bool is_owned = false;
public:
    {0}() = delete;
    {0}(void* self, bool is_owned) : self(self), is_owned(is_owned) {{ }}
    {0}({0}&& a) : self(a.self), is_owned(a.is_owned) {{ a.self = nullptr; a.is_owned = false; }};
    ~{0}() {{ if(this->self && this->is_owned) {{ __swift_bridge__${0}$_free(this->self); }} }};
    {1} unwrap() {{ return {1}(__swift_bridge__${0}$unwrap(this->self), true); }}
    bool is_some() {{ return __swift_bridge__${0}$is_some(this->self); }}
    void* get_ptr() {{ return this->self; }}
}};\n",
            $name, $inner_type
        )
    }};
}

macro_rules! result_class {
    ($name:ident, $inner_type:ident) => {
        format!(
            "
class {0} {{
    void* self = nullptr;
    bool is_owned = false;
public:
    {0}() = delete;
    {0}({0}&& a) : self(a.self), is_owned(a.is_owned) {{ a.self = nullptr; a.is_owned = false; }};
    {0}(void* self, bool is_owned) : self(self), is_owned(is_owned) {{ }}
    ~{0}() {{ if(this->self && this->is_owned) {{ __swift_bridge__${0}$_free(this->self); }} }};
    {1} unwrap() {{ return {1}(__swift_bridge__${0}$unwrap(this->self), true); }}
    ErrorType unwrap_err() {{ return ErrorType(__swift_bridge__${0}$unwrap_err(this->self), true); }}
    bool is_ok() {{ return __swift_bridge__${0}$is_ok(this->self); }}
    void* get_ptr() {{ return this->self; }}
}};\n",
            $name, $inner_type
        )
    };
}

macro_rules! custom_class_declaration {
    ($name:ident, $functions_declaration:ident) => {
        format!(
            "
class {0} {{
    void* self = nullptr;
    bool is_owned = false;
public:
    {0}() = delete;
    {0}(void* self, bool is_owned) : self(self), is_owned(is_owned) {{ }}
    {0}({0}&& a) : self(a.self), is_owned(a.is_owned) {{ a.self = nullptr; a.is_owned = false; }};
    ~{0}() {{ if(this->self && this->is_owned) {{ __swift_bridge__${0}$_free(this->self); }} }};
    void* get_ptr() {{ return this->self; }}
{1}}};
\n",
            $name, $functions_declaration
        )
    };
}

struct FunctionHelper {
    pub function_name: String,
    pub generated_args: String,
    pub generated_call: String,
    pub return_type_string: String,
    pub return_type: Option<WrapperType>,
}

fn map_function_to_helper_elements(function: &Function, skip_first: bool) -> FunctionHelper {
    let generated_args = function
        .arguments
        .iter()
        .skip(skip_first as usize)
        .map(|arg| match &arg.typ {
            WrapperType {
                typ: RustWrapperType::Vector,
                inner_type: Some(inner_type),
                ..
            } => {
                format!("RustVec<{}>& {}, ", inner_type.wrapper_name, arg.arg_name)
            }
            _ => {
                format!("{}& {}, ", arg.typ.wrapper_name, arg.arg_name)
            }
        })
        .collect::<String>();
    let generated_call = function
        .arguments
        .iter()
        .skip(skip_first as usize)
        .map(|arg| format!("{}.get_ptr(), ", arg.arg_name))
        .collect::<String>();
    let generated_call = if skip_first {
        format!("self, {}", generated_call)
    } else {
        generated_call
    };
    let return_type_string = function
        .return_type
        .as_ref()
        .map(|return_type| return_type.wrapper_name.to_string())
        .unwrap_or_else(|| "void".to_owned());
    let function_name = function.parsed_items.sig.ident.to_string();
    FunctionHelper {
        function_name,
        generated_args,
        generated_call,
        return_type_string,
        return_type: function.return_type.clone(),
    }
}

fn generate_functions_declaration(vec_of_functions_elems: &[FunctionHelper]) -> String {
    vec_of_functions_elems
        .iter()
        .map(
            |FunctionHelper {
                 function_name,
                 generated_args,
                 generated_call: _,
                 return_type_string,
                 return_type,
             }| {
                match return_type {
                    Some(WrapperType {
                        typ: RustWrapperType::Vector,
                        inner_type: Some(inner),
                        ..
                    }) => {
                        let inner_name = &inner.wrapper_name;
                        format!("    RustVec<{inner_name}> {function_name}({generated_args});\n")
                    }
                    _ => {
                        format!("    {return_type_string} {function_name}({generated_args});\n")
                    }
                }
            },
        )
        .collect::<String>()
}

fn generate_functions_definition(
    vec_of_functions_elems: &[FunctionHelper],
    class_name: Option<String>,
) -> String {
    let class_name_path = if let Some(ref class_name) = class_name {
        format!("{class_name}::")
    } else {
        "".to_owned()
    };
    let class_function_name = if let Some(class_name) = class_name {
        format!("{class_name}$")
    } else {
        "".to_owned()
    };
    // TODO: There's commented code below that should indicate a vector destructor method.
    //       Unfortunatelly vectors of primitives have methods `_free`
    //       while other types have methods `drop`. This code should distinguish
    //       between them.
    vec_of_functions_elems.iter().map(
        |FunctionHelper { function_name, generated_args, generated_call, return_type_string, return_type }| {
                match return_type {
                    Some(WrapperType {typ: RustWrapperType::Vector, inner_type: Some(inner), .. }) => {
                        let inner_name = inner.wrapper_name.to_string();
                        let inner_function_name = if inner_name == "String" {
                            "RustString".to_owned()
                        } else {
                            inner_name.clone()
                        };
                        format!(
                            "RustVec<{inner_name}> {class_name_path}{function_name}({generated_args}) {{
    return
        RustVec<{inner_name}>(
            __swift_bridge__${class_function_name}{function_name}({generated_call}),
            nullptr,  // TODO: __swift_bridge__$Vec_{inner_function_name}$drop
            __swift_bridge__$Vec_{inner_function_name}$push,
            __swift_bridge__$Vec_{inner_function_name}$get,
            __swift_bridge__$Vec_{inner_function_name}$len);
}}\n"
                        )
                    }
                    _ => {
                        format!(
                            "{return_type_string} {class_name_path}{function_name}({generated_args}) {{
    return {return_type_string}(__swift_bridge__${class_function_name}{function_name}({generated_call}), true);
}}\n"
                        )
                    }
                }
        }).collect::<String>()
}

pub fn generate_cpp_interface_file(
    extern_module_translator: &ExternModuleTranslator,
) -> GeneratedFilesContent {
    let classes_with_functions: HashSet<_> = extern_module_translator
        .structures_wrappers
        .keys()
        .cloned()
        .collect();
    let empty_types: String = extern_module_translator
        .rust_types_wrappers
        .difference(&classes_with_functions)
        .map(|wrapper| match wrapper {
            WrapperType {
                typ: RustWrapperType::Custom,
                ..
            } => {
                let class_name = wrapper.wrapper_name.to_string();
                let functions = "";
                custom_class_declaration!(class_name, functions)
            }
            _ => "".to_owned(),
        })
        .collect();

    let class_elements = extern_module_translator
        .structures_wrappers
        .iter()
        .map(|(wrapper_type, vec_of_functions)| {
            let function_elements = vec_of_functions
                .iter()
                .map(|f| map_function_to_helper_elements(f, true))
                .collect::<Vec<_>>();
            (wrapper_type, function_elements)
        })
        .collect::<Vec<_>>();

    let classes_declaration = class_elements
        .iter()
        .map(|(wrapper_type, vec_of_functions_elems)| {
            let functions_declaration = generate_functions_declaration(vec_of_functions_elems);
            let class_name = wrapper_type.wrapper_name.to_string();
            custom_class_declaration!(class_name, functions_declaration)
        })
        .collect::<String>();

    let classes_definition = class_elements
        .iter()
        .map(|(wrapper_type, vec_of_functions_elems)| {
            let class_name = wrapper_type.wrapper_name.to_string();
            generate_functions_definition(vec_of_functions_elems, Some(class_name))
        })
        .collect::<String>();

    let rust_types_wrappers: String = extern_module_translator
        .rust_types_wrappers
        .iter()
        .map(|wrapper| {
            let class_name = &wrapper.wrapper_name;
            match wrapper {
                WrapperType {
                    typ: RustWrapperType::Option,
                    inner_type: Some(inner),
                    ..
                } => {
                    let inner_type = inner.as_ref().wrapper_name.to_string();
                    option_class!(class_name, inner_type)
                }
                WrapperType {
                    typ: RustWrapperType::Result,
                    inner_type: Some(inner),
                    ..
                } => {
                    let inner_type = inner.as_ref().wrapper_name.to_string();
                    result_class!(class_name, inner_type)
                }
                _ => "".to_owned(),
            }
        })
        .collect();

    let global_functions_elems = extern_module_translator
        .global_functions
        .iter()
        .map(|f| map_function_to_helper_elements(f, false))
        .collect::<Vec<_>>();
    let global_functions_declaration = generate_functions_declaration(&global_functions_elems);
    let global_functions_definition = generate_functions_definition(&global_functions_elems, None);

    let templates: String = extern_module_translator
        .rust_types_wrappers
        .iter()
        .map(|key| match key.typ {
            RustWrapperType::Vector => format!(
                "%template(Vec{0}) RustVec<{0}>;\n",
                key.inner_type
                    .as_ref()
                    .expect("Vector has to have inner generic type")
                    .wrapper_name,
            ),
            _ => "".to_owned(),
        })
        .collect();
    GeneratedFilesContent {
        cpp_header: format!(
            "
{PREDEFINED}
{empty_types}
{classes_declaration}
{global_functions_declaration}
{rust_types_wrappers}
{global_functions_definition}
{classes_definition}
    "
        )
        .replace(", )", ")"),
        swig_interface: templates,
    }
}
