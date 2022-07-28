
class main {
    public static void main(java.lang.String argv[]) {
        System.out.println("Java FFI Test Suite");
        System.loadLibrary("wildland");

        var admin_manager = wildland.create_admin_manager(new RustString("lss.yaml")).unwrap();
        var user_api = admin_manager.user_api();
        var mnemonic = user_api.generate_mnemonic().unwrap();
        System.out.println(mnemonic.get_string().to_string());
        user_api.create_user_from_mnemonic(mnemonic, new RustString("My Mac")).unwrap();
        System.out.println("User successfully created from mnemonic");
    }
}

