print("Swift FFI Test Suite");

var admin_manager = create_admin_manager(RustString("lss.yaml")).unwrap();
var user_api = admin_manager.user_api();
var mnemonic = user_api.generate_mnemonic().unwrap();
print(mnemonic.get_string().to_string());
user_api.create_user_from_mnemonic(mnemonic, RustString("My Mac")).unwrap();
print("User successfully created from mnemonic");