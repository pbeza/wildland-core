using System;
using System.Collections.Generic;
using static wildland;

namespace Main
{
    class LocalSecureStorageImpl : LocalSecureStorage {
        private Dictionary <string, Vecu8> store = new Dictionary<string, Vecu8>();

        /// Inserts a key-value pair into the LSS.
        /// If the map did not have this key present, None is returned.
        /// If the map did have this key present, the value is updated, and the old value is returned.
        public override LssOptionalBytesResult insert(RustString key, Vecu8 value)
        {
            var std_key = key.to_string();
            LssOptionalBytesResult result;
            if (store.ContainsKey(std_key))
            {
                result = new_ok_lss_optional_bytes(new_some_bytes(store[std_key]));
            }
            else
            {
                result = new_ok_lss_optional_bytes(new_none_bytes());
            }
            store[std_key] = value;
            return result;
            // return new_err_lss_optional_bytes(new RustString("Err")); // EXAMPLE: returning error
        }

        /// Returns a copy of the value corresponding to the key.
        public override LssOptionalBytesResult get(RustString key)
        {
            var std_key = key.to_string();
            if (store.ContainsKey(std_key))
            {
                return new_ok_lss_optional_bytes(new_some_bytes(store[std_key]));
            }
            else
            {
                return new_ok_lss_optional_bytes(new_none_bytes());
            }
        }

        /// Returns true if the map contains a value for the specified key.
        public override LssBoolResult contains_key(RustString key)
        {
            var std_key = key.to_string();
            return new_ok_lss_bool(store.ContainsKey(std_key));
        }

        /// Returns all keys in arbitrary order.
        public override LssVecOfStringsResult keys()
        {
            VecRustString keys = new VecRustString();
            foreach(KeyValuePair<string, Vecu8> entry in store)
            {
                keys.push(new RustString(entry.Key));
            }
            return new_ok_lss_vec_of_strings(keys);
        }

        /// Removes a key from the map, returning the value at the key if the key was previously in the map.
        public override LssOptionalBytesResult remove(RustString key)
        {
            var std_key = key.to_string();
            LssOptionalBytesResult result;
            if (store.ContainsKey(std_key))
            {
                result = new_ok_lss_optional_bytes(new_some_bytes(store[std_key]));
                store.Remove(std_key);
            }
            else
            {
                result = new_ok_lss_optional_bytes(new_none_bytes());
            }
            return result;
        }

        /// Returns the number of elements in the map.
        public override LssUsizeResult len()
        {
            var len = (uint)store.Count;
            return new_ok_lss_usize(len);
        }

        /// Returns true if the map contains no elements, false otherwise.
        public override LssBoolResult is_empty()
        {
            return new_ok_lss_bool(store.Count == 0);
        }
    }

    class Program
    {
        static void Main(string[] args)
        {
            Console.WriteLine("C# FFI Test Suite");
            var cargo_lib = wildland.create_cargo_lib(new LocalSecureStorageImpl());
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