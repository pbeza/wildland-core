print("Swift FFI Test Suite");

var admin_manager = create_admin_manager();
var seed_phrase_result = admin_manager.create_seed_phrase();
if seed_phrase_result.is_ok() {
    var seed = seed_phrase_result.unwrap();
    var identities_result = admin_manager.create_wildland_identities(seed, RustString("device name 1"));
    var identity_pair = identities_result.unwrap();
    var forest_id = identity_pair.forest_id();
    var device_id = identity_pair.device_id();

    print(device_id.get_name().toString());
    device_id.set_name(RustString("name 2"));
    print(device_id.get_name().toString());
    print(device_id.to_string().toString());
    print(device_id.get_fingerprint_string().toString());
    print(device_id.get_private_key());

    var device_id_type = device_id.get_type();
    var another_device_id_type = device_id.get_type();
    var forest_id_type = forest_id.get_type();
    if (device_id_type.is_same(another_device_id_type)) {
      print("Types are equal");
    }
    if (!forest_id_type.is_same(device_id_type)) {
      print("Types are not equal");
    }
    if (!device_id_type.is_forest()) {
      print("it is not a forest type");
    }
    if (device_id_type.is_device()) {
      print("it is a device type");
    }

    if (device_id.save().is_ok()) {
      print("Device identity saved in a file.");
    }

    admin_manager.set_email(RustString("test@email.com"));
    var sending_result = admin_manager.request_verification_email(); // Code is hardcoded for now
    if (sending_result.is_ok()) {
      var verification_result = admin_manager.verify_email(RustString("123456"));
      if (verification_result.is_ok()) {
        print("Verification successful");
      } else {
        print(verification_result.unwrap_err().to_string().toString());
      }
    }
}