using System;

namespace Main
{
    class Program
    {
        static void Main(string[] args)
        {
            Console.WriteLine("C# FFI Test Suite");

            var admin_manager = wildland.create_admin_manager();
            var seed_result = admin_manager.create_seed_phrase();
            if (seed_result.is_ok()) {
                var seed = seed_result.unwrap();
                var identities_result = admin_manager.create_wildland_identities(seed, new RustString("device name 1"));
                var identity_pair = identities_result.unwrap();
                var forest_id = identity_pair.forest_id();
                var device_id = identity_pair.device_id();

                Console.WriteLine(device_id.get_name().c_str());
                device_id.set_name(new RustString("name 2"));
                Console.WriteLine(device_id.get_name().c_str());
                Console.WriteLine(device_id.to_string().c_str());
                Console.WriteLine(device_id.get_fingerprint_string().c_str());
                Console.WriteLine(device_id.get_private_key());

                var device_id_type = device_id.get_type();
                var another_device_id_type = device_id.get_type();
                var forest_id_type = forest_id.get_type();
                if (device_id_type.is_same(another_device_id_type)) {
                    Console.WriteLine("Types are equal");
                }
                if (!forest_id_type.is_same(device_id_type)) {
                    Console.WriteLine("Types are not equal");
                }
                if (!device_id_type.is_forest()) {
                    Console.WriteLine("it is not a forest type");
                }
                if (device_id_type.is_device()) {
                    Console.WriteLine("it is a device type");
                }
                
                if (device_id.save().is_ok()) {
                    Console.WriteLine("Device identity saved in a file.");
                }
            }


            admin_manager.set_email(new RustString("test@email.com"));
            var sending_result = admin_manager.request_verification_email();
            if (sending_result.is_ok()) {
                var verification_result = admin_manager.verify_email(new RustString("123456"));
                if (verification_result.is_ok()) {
                    Console.WriteLine("Verification successful");
                } else {
                    Console.WriteLine(verification_result.unwrap_err().to_string().c_str());
                }
            }
        }
    }
}