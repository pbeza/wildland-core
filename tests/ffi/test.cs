using System;

namespace Main
{
    class Program
    {
        static void Main(string[] args)
        {
            Console.WriteLine("C# FFI Test Suite");
            var cargo_lib = wildland.create_cargo_lib(new RustString("lss.yaml"));
            var user_api = cargo_lib.user_api();
            var mnemonic = user_api.generate_mnemonic();
            Console.WriteLine(mnemonic.get_string().to_string());

            user_api.create_user_from_mnemonic(mnemonic, new RustString("My Mac"));
            Console.WriteLine("User successfully created from mnemonic");

            var user = user_api.get_user();
            Console.WriteLine("User: " + user.get_string().to_string());
        }
    }
}