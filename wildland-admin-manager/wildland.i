%module wildland

// The following lines will be added to generated wrapper file.
%{
#include "mod.rs.h"
#include "cxx.h"
%}


// Translate C++ size_t to something more common to other langs
// in order to start using vectors (actually to get elements at
// given index)
%inline %{
typedef long unsigned int size_t;
typedef unsigned char uint8_t;
%}


// The is_complete structure can't be parsed by SWIG, so let's omit this
// during the process
%define CXXBRIDGE1_IS_COMPLETE; %enddef

// We don't want to have default constructors in our FFI API,
// since Opaque types cannot be created using cxx.rs
%nodefaultctor;

// Rename String to RustString, since there's already defined
// String class in Java and C#
%rename(RustString) String;

// Ignore unused cxx.rs structs internal structs (that are actually
// problematic during the parsing process for SWIG).
%ignore unsafe_bitcopy_t;
%ignore Opaque;
%ignore layout;


// We don't use this constructors, so it's better not to generate
// unused code and opaque types in the target languages.
%ignore cbegin;    // It's not that simple to provide users with iterators
%ignore cend;
%ignore begin;
%ignore end;
%ignore data;
%ignore Vec(unsafe_bitcopy_t, const Vec &);
%ignore Vec(std::initializer_list<T>);
%ignore String(unsafe_bitcopy_t, const String &);
%ignore String(const char *, std::size_t);
%ignore String(const char16_t *, std::size_t);

// We can't create and copy Opaque types in CXX.RS, but we want to
// return Box<T>, where T is an Opaque type. Hence we have to drop
// the Box copy and move constructors in order to use Boxes in 
// the target languages.
%ignore Box(const T &);
%ignore Box(T &&);

// For Rust Box support:
// The following typemap introduce move semantics in the generated code.
// It is necessary when one need to create a type returned by value
// from Rust and which doesn't support copy constructor (like Box).
// TODO: Extend the comment to this section
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
%include "mod.rs.h"


// We have to instantiate templates that we use.
// TODO: Extend the comment to this section with some instructions.
// %template(StringVector) ::rust::cxxbridge1::Vec<::rust::cxxbridge1::String>;
// %template(ByteVector) ::rust::cxxbridge1::Vec<::std::uint8_t>;
%template(BoxedSeedPhraseResult) ::rust::cxxbridge1::Box<::wildland::SeedPhraseResult>;
%template(BoxedIdentityResult) ::rust::cxxbridge1::Box<::wildland::IdentityResult>;
%template(BoxedOptionalResult) ::rust::cxxbridge1::Box<::wildland::OptionalIdentity>;
%template(BoxedAdminManager) ::rust::cxxbridge1::Box<::wildland::CxxAdminManager>;
