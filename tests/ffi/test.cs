using System;

namespace Main
{
    class Program
    {
        static void Main(string[] args)
        {
            Console.WriteLine("C# FFI Test Suite");
            var admin_manager = wildland.create_admin_manager(new RustString("/tmp/lss.yaml"));
            var mnemonic_result = admin_manager.user_api().generate_mnemonic();
            if (mnemonic_result.is_ok()) {
                var mnemonic = mnemonic_result.unwrap();
                Console.WriteLine(mnemonic.get_string().to_string());
            }
        }
    }
}
