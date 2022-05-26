using System;

namespace Main
{
    class Program
    {
        static void Main(string[] args)
        {
            Console.WriteLine("TODO: C# FFI Testsuite");
        
            var admin_manager = wildland.create_admin_manager();
            var seed_result = wildland.create_seed_phrase();
            if (seed_result.is_ok()) {
                var seed = seed_result.unwrap();
                var identity_result = admin_manager.create_master_identity_from_seed_phrase(new RustString("name 1"), seed);
                var identity = identity_result.unwrap();
                Console.WriteLine(identity.get_name().c_str());
                identity.set_name(new RustString("name 2"));
                Console.WriteLine(identity.get_name().c_str());
            }
        }
    }
}