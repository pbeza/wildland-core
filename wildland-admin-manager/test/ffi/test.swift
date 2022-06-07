print("TODO: Swift FFI Test Suite");

var email_client_mock_builder = create_email_client_mock_builder();
email_client_mock_builder.expect_send(RustString("test@email.com"), RustString("123456"), 1);
var email_client_mock = email_client_mock_builder.build();

var seed_phrase_result = create_seed_phrase();
var admin_manager = create_admin_manager(email_client_mock);
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

    admin_manager.set_email(RustString("test@email.com"));
    var sending_result = admin_manager.send_verification_code(); // Code is hardcoded for now
    if (sending_result.is_ok()) {
      var verification_result = admin_manager.verify_email(RustString("123456"));
      if (verification_result.is_ok()) {
        print("Verification successful");
      } else {
        print(verification_result.unwrap_err().to_string().toString());
      }
    }
}