using System;

namespace Main
{
    class Program
    {
        static void Main(string[] args)
        {
            Console.WriteLine("C# FFI Testsuite");
        
            var admin_manager = wildland.create_admin_manager();
            var seed_result = wildland.create_seed_phrase();
            if (seed_result.is_ok()) {
                var seed = seed_result.unwrap();
                var identity_result = admin_manager.create_master_identity_from_seed_phrase(new RustString("name 1"), seed);
                var identity = identity_result.unwrap();
                Console.WriteLine(identity.get_name().c_str());
                identity.set_name(new RustString("name 2"));
                Console.WriteLine(identity.get_name().c_str());
            
                var identity_opt = admin_manager.get_master_identity(); // second ref to the same identity
                if (identity_opt.is_some()) {
                    var identity_second_ref = identity_opt.unwrap();
                    identity_second_ref.set_name(new RustString("name 3"));
                    Console.WriteLine(identity_second_ref.get_name().c_str());
                }

                Console.WriteLine(identity.get_name().c_str()); // first ref is still valid
            }
        }
    }
}