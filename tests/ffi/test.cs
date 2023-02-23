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
        public override FoundationCloudMode get_foundation_cloud_env_mode() {
            return FoundationCloudMode.Dev;
        }
        public override RustString get_redis_url() {
            return new RustString("redis://127.0.0.1/0");
        }
    }

    class LocalSecureStorageImpl : LocalSecureStorage {
        private Dictionary <string, RustString> store = new Dictionary<string, RustString>();

        /// Inserts a key-value pair into the LSS.
        /// If the map did not have this key present, None is returned.
        /// If the map did have this key present, the value is updated, and the old value is returned.
        public override OptionalRustStringResultWithLssError insert(RustString key, RustString value)
        {
            var std_key = key.to_string();
            OptionalRustStringResultWithLssError result;
            if (store.ContainsKey(std_key))
            {
                result = OptionalRustStringResultWithLssError.from_ok(new OptionalRustString(store[std_key]));
            }
            else
            {
                result = OptionalRustStringResultWithLssError.from_ok(new OptionalRustString());
            }
            store[std_key] = value;
            return result;
            // return new_err_lss_optional_bytes(new RustString("Err")); // EXAMPLE: returning error
        }

        /// Returns a copy of the value corresponding to the key.
        public override OptionalRustStringResultWithLssError get(RustString key)
        {
            var std_key = key.to_string();
            if (store.ContainsKey(std_key))
            {
                return OptionalRustStringResultWithLssError.from_ok(new OptionalRustString(store[std_key]));
            }
            else
            {
                return OptionalRustStringResultWithLssError.from_ok(new OptionalRustString());
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
            foreach(KeyValuePair<string, RustString> entry in store)
            {
                keys.push(new RustString(entry.Key));
            }
            return VecRustStringResultWithLssError.from_ok(keys);
        }

        /// Removes a key from the map, returning the value at the key if the key was previously in the map.
        public override OptionalRustStringResultWithLssError remove(RustString key)
        {
            var std_key = key.to_string();
            OptionalRustStringResultWithLssError result;
            if (store.ContainsKey(std_key))
            {
                result = OptionalRustStringResultWithLssError.from_ok(new OptionalRustString(store[std_key]));
                store.Remove(std_key);
            }
            else
            {
                result = OptionalRustStringResultWithLssError.from_ok(new OptionalRustString());
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

            // -------------- TEMPLATES ----------------

            Console.WriteLine("TEST: Create and save storage template from json");
            var tpl_str = @"{
                ""template"": {
                    ""access"":[
                        {""user"":""*""}
                    ],
                    ""credentials"":{
                        ""access-key"":""NOT_SO_SECRET"",
                        ""secret-key"":""VERY_SECRET""
                    },
                    ""manifest-pattern"":{
                        ""path"":""/{path}.yaml"",
                        ""type"":""glob""
                    },
                    ""read-only"":true,
                    ""s3_url"":""s3://michal-afc03a81-307c-4b41-b9dd-771835617900/{{ CONTAINER_UUID  }}"",
                    ""with-index"":false
                },
                ""backend_type"":""s3""
            }";
            var json_tpl = new Vecu8();

            foreach(byte b in System.Text.Encoding.UTF8.GetBytes(tpl_str)) { json_tpl.push(b); }

            var tpl = wildland.storage_template_from_json(json_tpl);
            tpl.set_name(new RustString("Some JSON template"));
            var tpl_uuid = user.save_storage_template(tpl);
            Console.WriteLine($"[OK] Storage Template saved with uuid: {tpl_uuid.to_string()}");
            Console.WriteLine($"Serialized Template: {tpl.to_json().to_string()}");

            Console.WriteLine("TEST: Create and save storage template from yaml");
            tpl_str = @"---
template:
    access:
        - user: '*'
    credentials:
        access-key: NOT_SO_SECRET
        secret-key: VERY_SECRET
    manifest-pattern:
        path: /{path}.yaml
        type: glob
    read-only: true
    s3_url: s3://michal-afc03a81-307c-4b41-b9dd-771835617900/{{ CONTAINER_UUID  }}
    with-index: false
backend_type: s3
";

            var yaml_tpl = new Vecu8();

            foreach(byte b in System.Text.Encoding.UTF8.GetBytes(tpl_str)) { yaml_tpl.push(b); }

            tpl = wildland.storage_template_from_yaml(yaml_tpl);
            tpl.set_name(new RustString("Some YAML template"));
            tpl_uuid = user.save_storage_template(tpl);
            Console.WriteLine($"[OK] Storage Template saved with uuid: {tpl_uuid.to_string()}");
            Console.WriteLine($"Serialized Template: {tpl.to_yaml().to_string()}");
        }
    }
}
