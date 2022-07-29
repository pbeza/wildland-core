using System;

namespace Main
{
    class Program
    {
        static void Main(string[] args)
        {
            Console.WriteLine("C# FFI Test Suite");
            var admin_manager = wildland.create_admin_manager(new RustString("lss.yaml")).unwrap();
            var user_api = admin_manager.user_api();
            var mnemonic = user_api.generate_mnemonic().unwrap();
            Console.WriteLine(mnemonic.get_string().to_string());

            user_api.create_user_from_mnemonic(mnemonic, new RustString("My Mac")).unwrap();
            Console.WriteLine("User successfully created from mnemonic");

            var user = user_api.get_user().unwrap();
            Console.WriteLine("User: " + user);
        }
    }
}