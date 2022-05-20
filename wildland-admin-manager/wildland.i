%module wildland

// The following lines will be added to generated wrapper file.
%{
#include "lib.rs.h"
#include "cxx.h"
%}


// Translate c++ size_t to something more common to other langs
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
%nodefaultctor;

// Rename String to RustString, since there's already defined
// String class in Java
%rename(RustString) String;

// Inlcude the C++ API
%include "lib.rs.h"

// We use Vec<String> and Vec<u8> so we have to instantiate Vec template
%template(StringVector) ::rust::cxxbridge1::Vec<::rust::cxxbridge1::String>;
%template(ByteVector) ::rust::cxxbridge1::Vec<::std::uint8_t>;

