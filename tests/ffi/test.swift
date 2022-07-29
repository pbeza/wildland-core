
// This test file is not supported since ffi-macro v.0.2.0

print("Swift FFI Test Suite");

var cargo_lib = create_cargo_lib(RustString("lss.yaml")).unwrap();
var mnemonic_result = cargo_lib.user_api().generate_mnemonic();
if mnemonic_result.is_ok() {
    var mnemonic = mnemonic_result.unwrap();
    print(mnemonic.get_string().to_string());
}
