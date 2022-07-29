
// This test file is not supported since ffi-macro v.0.2.0

print("Swift FFI Test Suite");

var cargo_lib = create_cargo_lib(RustString("lss.yaml")).unwrap();
var user_api = cargo_lib.user_api();
var mnemonic = user_api.generate_mnemonic().unwrap();
print(mnemonic.get_string().to_string());
user_api.create_user_from_mnemonic(mnemonic, RustString("My Mac")).unwrap();
print("User successfully created from mnemonic");