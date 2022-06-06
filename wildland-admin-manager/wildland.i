%module wildland

// The following lines will be added to generated wrapper file.
%{
#include "mod.rs.h"
#include "cxx.h"
%}


// Translate C++ specific numeric types to something more common 
// to other langs in order to start using vectors
%inline %{
typedef long unsigned int size_t;
typedef unsigned char uint8_t;
%}


// The is_complete structure can't be parsed by SWIG - this line 
// tells SWIG to omit this during the process
%define CXXBRIDGE1_IS_COMPLETE; %enddef

// We don't want to have default constructors in our FFI API,
// since CXX generated `Opaque` types cannot be created using
// on the target language side.
%nodefaultctor;

// Rename String to RustString, since there's already defined
// String class in Java and C#
%rename(RustString) String;

// Ignore unused cxx.rs internal structs (that are actually
// problematic during the parsing process for SWIG).
%ignore unsafe_bitcopy_t;
%ignore Opaque;
%ignore layout;

// The following methods cannot be used in a convinient form
// in the target languages, hence it is worth to ignore them
// so that SWIG will not create unnecessary unused boilerplate code.
// NOTE: The first methods comes from Iterator definition.
//       It's not clear how to provide end users with iterators
//       using SWIG yet.
%ignore cbegin;    
%ignore cend;
%ignore begin;
%ignore end;
%ignore data;
%ignore Vec(unsafe_bitcopy_t, const Vec &);
%ignore Vec(std::initializer_list<T>);
%ignore String(unsafe_bitcopy_t, const String &);
%ignore String(const char *, std::size_t);
%ignore String(const char16_t *, std::size_t);

// We cannot copy Opaque types in CXX.RS generated code, but we want to
// return Box<T>, where T is an Opaque type. Hence we have to ignore
// the Box copy and move constructors in order to use Boxes in 
// the target languages.
%ignore Box(const T &);
%ignore Box(T &&);

// For Rust Box support:
// The problem:
// Swig by default do some copying of the objects coming from the source
// language. Unfortunately Box<T>, where T is an Opaque type originated in CXX
// cannot be copied. Technically all rust custom types inherence from Opaque
// type in cxx generated code. The mentioned parent type has deleted destructor
// and constructors which means that no Opaque object can be copied, created or
// detroyed in the target language.
// The following SWIG typemaps introduces move semantics in the generated code
// instead of copying objects.
%typemap(java, out, optimal="1") SWIGTYPE %{
  $result = new $1_ltype(( $1_ltype &&)$1);
%}

%typemap(csharp, out, optimal="1") SWIGTYPE %{
  $result = new $1_ltype(( $1_ltype &&)$1);
%}

%typemap(python, out, optimal="1") SWIGTYPE %{
  $result = SWIG_NewPointerObj((new $1_ltype(static_cast< $1_ltype&&  >($1))), $&1_descriptor, SWIG_POINTER_OWN |  0 );
%}

//////////////////////////////////
// Inlcude the generated C++ API
//////////////////////////////////
%include "mod.rs.h"

// There are two generic types that we use on the FFI layer: Box<T> and Vec<T>
// Swig needs to have defined the concrete instances of templated types in
// order to let the target languages users handle them properly.
// In result each type added to cxx module that uses generics needs to
// be declared here:
%template(BoxedSeedPhraseResult) ::rust::cxxbridge1::Box<::wildland::SeedPhraseResult>;
%template(BoxedSeedPhrase) ::rust::cxxbridge1::Box<::wildland::SeedPhrase>;
%template(BoxedDynIdentity) ::rust::cxxbridge1::Box<::wildland::DynIdentity>;
%template(BoxedIdentityResult) ::rust::cxxbridge1::Box<::wildland::IdentityResult>;
%template(BoxedOptionalIdentity) ::rust::cxxbridge1::Box<::wildland::OptionalIdentity>;
%template(BoxedAdminManager) ::rust::cxxbridge1::Box<::wildland::AdminManager>;
%template(BoxedEmptyResult) ::rust::cxxbridge1::Box<::wildland::EmptyResult>;
%template(BoxedAdminManagerError) ::rust::cxxbridge1::Box<::wildland::AdminManagerError>;
