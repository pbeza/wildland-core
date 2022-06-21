
class main {
  public static void main(java.lang.String argv[]) {
    System.out.println("Java FFI Test Suite");
    System.loadLibrary("wildland");

    var admin_manager = wildland.create_admin_manager();
    var seed_result = admin_manager.create_seed_phrase();
    if (seed_result.is_ok()) {
      var seed = seed_result.unwrap();
      var identities_result = admin_manager.create_wildland_identities(seed, new RustString("device name 1"));
      var identity_pair = identities_result.unwrap();
      var forest_id = identity_pair.forest_id();
      var device_id = identity_pair.device_id();

      System.out.println(device_id.get_name().c_str());
      device_id.set_name(new RustString("name 2"));
      System.out.println(device_id.get_name().c_str());
      System.out.println(device_id.to_string().c_str());
      System.out.println(device_id.get_fingerprint_string().c_str());
      System.out.println(device_id.get_private_key());
    }

    admin_manager.set_email(new RustString("test@email.com"));
    var sending_result = admin_manager.request_verification_email();
    if (sending_result.is_ok()) {
      var verification_result = admin_manager.verify_email(new RustString("123456"));
      if (verification_result.is_ok()) {
        System.out.println("Verification successful");
      } else {
        System.out.println(verification_result.unwrap_err().to_string().c_str());
      }
    }
  }
}
