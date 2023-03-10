
class main {
    public static void main(java.lang.String argv[]) {
        System.out.println("Java FFI Test Suite");
        System.loadLibrary("wildland");

        var cargo_lib = wildland.create_cargo_lib(new RustString("lss.yaml")).unwrap();
        var user_api = cargo_lib.user_api();
        var mnemonic = user_api.generate_mnemonic().unwrap();
        System.out.println(mnemonic.stringify().to_string());
        user_api.create_user_from_mnemonic(mnemonic, new RustString("My Mac")).unwrap();
        System.out.println("User successfully created from mnemonic");
        var user = user_api.get_user().unwrap().unwrap();
        System.out.println("User: " + user.stringify().to_string());
    }
}

