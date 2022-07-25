
class main {
  public static void main(java.lang.String argv[]) {
    System.out.println("Java FFI Test Suite");
    System.loadLibrary("wildland");

    var cargo_lib = wildland.create_cargo_lib();
    var mnemonic_result = cargo_lib.user_api().generate_mnemonic();
    if (mnemonic_result.is_ok()) {
      var mnemonic = mnemonic_result.unwrap();
      System.out.println(mnemonic.get_string().to_string());
    }
  }
}
