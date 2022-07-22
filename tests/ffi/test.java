
class main {
  public static void main(java.lang.String argv[]) {
    System.out.println("Java FFI Test Suite");
    System.loadLibrary("wildland");

    var admin_manager = wildland.create_admin_manager(new RustString("/tmp/lss.yaml"));
    var mnemonic_result = admin_manager.user_api().generate_mnemonic();
    if (mnemonic_result.is_ok()) {
      var mnemonic = mnemonic_result.unwrap();
      System.out.println(mnemonic.get_string().to_string());
    }
  }
}
