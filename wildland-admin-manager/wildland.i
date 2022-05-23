%module wildland

// The following lines will be added to generated wrapper file.
%{
#include "ffi.rs.h"
#include "cxx.h"
%}


// Translate C++ size_t to something more common to other langs
// in order to start using vectors (actually to get elements at
// given index)
%inline %{
typedef long unsigned int size_t;
typedef unsigned char uint8_t;
%}


// The is_complete structure is not parsed by SWIG, so omit this
// during the process
%define CXXBRIDGE1_IS_COMPLETE;
%enddef

// We don't want to have default constructors in our FFI API
// Since Opaque types cannot be created using cxx.rs
%nodefaultctor;

// Rename String to RustString, since there's already defined
// String class in Java
%rename(RustString) String;

// Ignore unused cxx.rs structs
%ignore unsafe_bitcopy_t;
%ignore Opaque;
%ignore layout;

// We can't create Opaque types in cxx.rs
%ignore Box(const T &);
%ignore Box(T &&);

// For Rust Box support:
// The following typemap introduce move semantics in the generated code.
// It is necessary when one need to create a type returned by value
// from Rust and which doesn't support copy constructor (like Box).
%typemap(java, out, optimal="1") SWIGTYPE %{
  $result = new $1_ltype(( $1_ltype &&)$1);
%}

%typemap(csharp, out, optimal="1") SWIGTYPE %{
  $result = new $1_ltype(( $1_ltype &&)$1);
%}

%typemap(python, out, optimal="1") SWIGTYPE %{
  $result = SWIG_NewPointerObj((new $1_ltype(static_cast< $1_ltype&&  >($1))), $&1_descriptor, SWIG_POINTER_OWN |  0 );
%}

// Inlcude the generated C++ API
%include "ffi.rs.h"


// We have to instantiate templates that we use.
%template(StringVector) ::rust::cxxbridge1::Vec<::rust::cxxbridge1::String>;
%template(ByteVector) ::rust::cxxbridge1::Vec<::std::uint8_t>;
%template(AdminRefsVecBoxed) ::rust::cxxbridge1::Box<::wildland::adminmanager::RcRefAdminManager>;
%template(ArrayAdminManagerBoxed) ::rust::cxxbridge1::Box<::wildland::adminmanager::ArrayAdminManager>;
%template(AdminRefs) ::rust::cxxbridge1::Box<::wildland::adminmanager::RcRefAdminManager>;
