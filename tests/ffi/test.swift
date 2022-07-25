print("Swift FFI Test Suite");

var cargo_lib = create_cargo_lib();
var mnemonic_result = cargo_lib.user_api().generate_mnemonic();
if mnemonic_result.is_ok() {
    var mnemonic = mnemonic_result.unwrap();
    print(mnemonic.get_string().to_string());
}