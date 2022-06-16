%module wildland


// // Rename String to RustString, since there's already defined
// // String class in Java and C#
%rename(RustString) String;

// The problem:
// Swig by default do some copying of the objects coming from the source
// language. Since swift-bridge creates each opaque type inside a Box<T>, those
// cannot be copied. If we try to for e.g. return such an element by a copy
// then the desctructor is called on the source object - the second one becomes
// invalidated immadietely.
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
%{
  extern "C" {
    #include "ffi_swift.h"
    #include "SwiftBridgeCore.h"
  }
  #include "ffi_cxx.h"
%}
%include "./_generated_cpp/ffi_cxx.h"
%include "./_generated_cpp/ffi_swig.i"
