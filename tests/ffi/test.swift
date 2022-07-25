print("Swift FFI Test Suite");

var admin_manager = create_admin_manager(RustString("lss.yaml")).unwrap();
var mnemonic_result = admin_manager.user_api().generate_mnemonic();
if mnemonic_result.is_ok() {
    var mnemonic = mnemonic_result.unwrap();
    print(mnemonic.get_string().to_string());
}