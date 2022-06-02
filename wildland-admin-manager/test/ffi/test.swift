print("TODO: Swift FFI Testsuite");

var seed_phrase_result = create_seed_phrase();
var admin_manager = create_admin_manager();
if seed_phrase_result.is_ok() {
    var seed = seed_phrase_result.unwrap();
    var identity_result = admin_manager.create_master_identity_from_seed_phrase(RustString("name 1"), seed);
    var identity = identity_result.unwrap();
    print(identity.get_name().toString())
    identity.set_name(RustString("name 2"));
    print(identity.get_name().toString())

    var identity_opt = admin_manager.get_master_identity(); // second ref to the same identity
    if (identity_opt.is_some()) {
        var identity_second_ref = identity_opt.unwrap();
        identity_second_ref.set_name(RustString("name 3"));
        print(identity_second_ref.get_name().toString());
    }

    print(identity.get_name().toString()); // first ref is still valid
}