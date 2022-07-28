using System;

namespace Main
{
    class Program
    {
        static void Main(string[] args)
        {
            Console.WriteLine("C# FFI Test Suite");
            var cargo_lib = wildland.create_cargo_lib(new RustString("lss.yaml")).unwrap();
            var mnemonic_result = cargo_lib.user_api().generate_mnemonic();
            if (mnemonic_result.is_ok()) {
                var mnemonic = mnemonic_result.unwrap();
                Console.WriteLine(mnemonic.get_string().to_string());
            }
        }
    }
}
