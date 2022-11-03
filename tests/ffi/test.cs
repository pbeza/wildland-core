using System;
using System.Collections.Generic;
using static wildland;

namespace Main
{
    class CargoCfgProviderImpl : CargoCfgProvider {
        public override bool get_use_logger() {
            return true;
        }
        public override RustString get_log_level() {
            return new RustString("debug");
        }
        public override bool get_log_use_ansi() {
            return false;
        }
        public override bool get_log_file_enabled() {
            return true;
        }
        public override OptionalRustString get_log_file_path() {
            return new OptionalRustString();
        }
        public override OptionalRustString get_log_file_rotate_directory() {
            return new OptionalRustString();
        }
        public override OptionalRustString get_oslog_category() {
            return new OptionalRustString();
        }
        public override OptionalRustString get_oslog_subsystem() {
            return new OptionalRustString();
        }

        public override FoundationCloudMode get_foundation_cloud_env_mode() {
            return FoundationCloudMode.Dev;
        }
    }

    class LocalSecureStorageImpl : LocalSecureStorage {
        private Dictionary <string, Vecu8> store = new Dictionary<string, Vecu8>();

        /// Inserts a key-value pair into the LSS.
        /// If the map did not have this key present, None is returned.
        /// If the map did have this key present, the value is updated, and the old value is returned.
        public override OptionalVecu8ResultWithLssError insert(RustString key, Vecu8 value)
        {
            var std_key = key.to_string();
            OptionalVecu8ResultWithLssError result;
            if (store.ContainsKey(std_key))
            {
                result = OptionalVecu8ResultWithLssError.from_ok(new OptionalVecu8(store[std_key]));
            }
            else
            {
                result = OptionalVecu8ResultWithLssError.from_ok(new OptionalVecu8());
            }
            store[std_key] = value;
            return result;
            // return new_err_lss_optional_bytes(new RustString("Err")); // EXAMPLE: returning error
        }

        /// Returns a copy of the value corresponding to the key.
        public override OptionalVecu8ResultWithLssError get(RustString key)
        {
            var std_key = key.to_string();
            if (store.ContainsKey(std_key))
            {
                return OptionalVecu8ResultWithLssError.from_ok(new OptionalVecu8(store[std_key]));
            }
            else
            {
                return OptionalVecu8ResultWithLssError.from_ok(new OptionalVecu8());
            }
        }

        /// Returns true if the map contains a value for the specified key.
        public override boolResultWithLssError contains_key(RustString key)
        {
            var std_key = key.to_string();
            return boolResultWithLssError.from_ok(store.ContainsKey(std_key));
        }

        /// Returns all keys in arbitrary order.
        public override VecRustStringResultWithLssError keys()
        {
            VecRustString keys = new VecRustString();
            foreach(KeyValuePair<string, Vecu8> entry in store)
            {
                keys.push(new RustString(entry.Key));
            }
            return VecRustStringResultWithLssError.from_ok(keys);
        }

        /// Removes a key from the map, returning the value at the key if the key was previously in the map.
        public override OptionalVecu8ResultWithLssError remove(RustString key)
        {
            var std_key = key.to_string();
            OptionalVecu8ResultWithLssError result;
            if (store.ContainsKey(std_key))
            {
                result = OptionalVecu8ResultWithLssError.from_ok(new OptionalVecu8(store[std_key]));
                store.Remove(std_key);
            }
            else
            {
                result = OptionalVecu8ResultWithLssError.from_ok(new OptionalVecu8());
            }
            return result;
        }

        /// Returns the number of elements in the map.
        public override usizeResultWithLssError len()
        {
            var len = (uint)store.Count;
            return usizeResultWithLssError.from_ok(len);
        }

        /// Returns true if the map contains no elements, false otherwise.
        public override boolResultWithLssError is_empty()
        {
            return boolResultWithLssError.from_ok(store.Count == 0);
        }
    }

    class Program
    {
        static void Main(string[] args)
        {
            Console.WriteLine("C# FFI Test Suite");
            var cargo_lib = wildland.create_cargo_lib(new LocalSecureStorageImpl(), wildland.collect_config(new CargoCfgProviderImpl()));
            var user_api = cargo_lib.user_api();
            var mnemonic = user_api.generate_mnemonic();
            Console.WriteLine(mnemonic.stringify().to_string());

            user_api.create_user_from_mnemonic(mnemonic, new RustString("My Mac"));
            Console.WriteLine("User successfully created from mnemonic");

            var user = user_api.get_user();
            Console.WriteLine("User: " + user.stringify().to_string());
        }
    }
}
