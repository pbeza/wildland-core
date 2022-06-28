
class main {
  public static void main(java.lang.String argv[]) {
    System.out.println("Java FFI Test Suite");
    System.loadLibrary("wildland");

    var admin_manager = wildland.create_admin_manager();
    var seed_result = wildland.create_seed_phrase();
    if (seed_result.is_ok()) {
      var seed = seed_result.unwrap();
      var identity_result = admin_manager.create_master_identity_from_seed_phrase(new RustString("name 1"), seed);
      var identity = identity_result.unwrap();
      System.out.println(identity.get_name().to_string());
      identity.set_name(new RustString("name 2"));
      System.out.println(identity.get_name().to_string());

      var identity_opt = admin_manager.get_master_identity(); // second ref to the same identity
      if (identity_opt.is_some()) {
        var identity_second_ref = identity_opt.unwrap();
        identity_second_ref.set_name(new RustString("name 3"));
        System.out.println(identity_second_ref.get_name().to_string());
      }

      System.out.println(identity.get_name().to_string()); // first ref is still valid
    }

    admin_manager.set_email(new RustString("test@email.com"));
    var sending_result = admin_manager.request_verification_email(); // Code is hardcoded for now
    if (sending_result.is_ok()) {
      var verification_result = admin_manager.verify_email(new RustString("123456"));
      if (verification_result.is_ok()) {
        System.out.println("Verification successful");
      } else {
        System.out.println(verification_result.unwrap_err().to_string().to_string());
      }
    }
  }
}
