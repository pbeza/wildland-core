#include <iostream>
#include <unordered_map>
#include <cassert>
#include "ffi_cxx.h"

class CargoCfgProviderImpl : public CargoCfgProvider
{
    bool get_use_logger() override
    {
        return true;
    }
    String get_log_level() override
    {
        return RustString("info");
    }
    bool get_log_use_ansi() override
    {
        return false;
    }
    bool get_log_file_enabled() override
    {
        return true;
    }
    Optional<String> get_log_file_path() override
    {
        return Optional<String>();
    }
    Optional<String> get_log_file_rotate_directory() override
    {
        return Optional<String>();
    }
    Optional<String> get_oslog_category() override
    {
        return Optional<String>();
    }
    Optional<String> get_oslog_subsystem() override
    {
        return Optional<String>();
    }
    FoundationCloudMode get_foundation_cloud_env_mode() override
    {
        return FoundationCloudMode::Dev;
    }
};

class LocalSecureStorageImpl : public LocalSecureStorage
{
    /// Inserts a key-value pair into the LSS.
    /// If the map did not have this key present, None is returned.
    /// If the map did have this key present, the value is updated, and the old value is returned.
    OptionalRustStringResultWithLssError insert(String key, RustString value) override
    {
        std::cout << "LSS insert C++ impl\n";
        auto std_key = key.to_string();
        OptionalRustStringResultWithLssError result;
        if (store.contains(std_key))
        {
            result = OptionalRustStringResultWithLssError::from_ok(Optional<RustString>(store[std_key]));
        }
        else
        {
            result = OptionalRustStringResultWithLssError::from_ok(Optional<RustString>());
        }
        store[std_key] = value;
        return result;
        // return new_err_lss_optional_bytes(String{"Err"}); // EXAMPLE: returning error
    }

    /// Returns a copy of the value corresponding to the key.
    OptionalRustStringResultWithLssError get(String key) override
    {
        std::cout << "LSS get C++ impl\n";
        auto std_key = key.to_string();
        if (store.contains(std_key))
        {
            return OptionalRustStringResultWithLssError::from_ok(Optional<RustString>(store[std_key]));
        }
        else
        {
            return OptionalRustStringResultWithLssError::from_ok(Optional<RustString>());
        }
    }

    /// Returns true if the map contains a value for the specified key.
    boolResultWithLssError contains_key(String key) override
    {
        std::cout << "LSS contains_key C++ impl\n";
        auto std_key = key.to_string();
        return boolResultWithLssError::from_ok(store.contains(std_key));
    }

    /// Returns all keys in arbitrary order.
    VecRustStringResultWithLssError keys() override
    {
        std::cout << "LSS keys C++ impl\n";
        RustVec<String> keys;
        for (const auto &[k, v] : store)
        {
            keys.push(String{k});
        }
        return VecRustStringResultWithLssError::from_ok(keys);
    }

    /// Returns all keys in arbitrary order.
    VecRustStringResultWithLssError keys_starting_with(RustString prefix) override
    {
        std::cout << "LSS keys C++ impl\n";
        RustVec<RustString> keys;
        auto prefix_str = prefix.to_string();
        for (const auto &[k, v] : store)
        {
            if (k.starts_with(prefix_str))
                keys.push(RustString{k});
        }
        return VecRustStringResultWithLssError::from_ok(keys);
    }

    /// Removes a key from the map, returning the value at the key if the key was previously in the map.
    OptionalRustStringResultWithLssError remove(String key) override
    {
        std::cout << "LSS remove C++ impl\n";
        auto std_key = key.to_string();
        OptionalRustStringResultWithLssError result;
        if (store.contains(std_key))
        {
            result = OptionalRustStringResultWithLssError::from_ok(Optional<RustString>(store[std_key]));
            store.erase(std_key);
        }
        else
        {
            result = OptionalRustStringResultWithLssError::from_ok(Optional<RustString>());
        }
        return result;
    }

    /// Returns the number of elements in the map.
    usizeResultWithLssError len() override
    {
        std::cout << "LSS len C++ impl\n";
        return usizeResultWithLssError::from_ok(store.size());
    }

    /// Returns true if the map contains no elements, false otherwise.
    boolResultWithLssError is_empty() override
    {
        std::cout << "LSS is_empty C++ impl\n";
        return boolResultWithLssError::from_ok(store.empty());
    }

private:
    std::unordered_map<std::string, RustString> store = {};
};

void config_parser_test() // test
{
    RustVec<u8> config_bytes{};
    std::string raw_config = "{\"log_level\": \"trace\", \"evs_url\": \"http://some_evs_endpoint/\"}";
    for (const auto ch : raw_config)
    {
        config_bytes.push(ch);
    }
    LocalSecureStorageImpl lss{};
    try
    {
        CargoConfig cargo_cfg = parse_config(config_bytes);
        SharedMutexCargoLib cargo_lib = create_cargo_lib(lss, cargo_cfg);
    }
    catch (const RustExceptionBase &e)
    {
        std::cout << e.reason().to_string() << std::endl;
    }
}

auto foundation_storage_test(CargoUser &cargo_user)
{
    std::cout << "is user onboard? " << std::boolalpha << cargo_user.is_free_storage_granted() << std::endl;

    auto process_handle = cargo_user.request_free_tier_storage("test@wildland.io");

    std::cout << "Provide a verification token:\n";
    std::string verification_token;
    std::cin >> verification_token;
    // may be used for creating container
    StorageTemplate storage_template = cargo_user.verify_email(process_handle, RustString{verification_token});
    std::cout << storage_template.stringify().to_string() << std::endl;

    std::cout << "is user onboard? " << std::boolalpha << cargo_user.is_free_storage_granted() << std::endl;

    return storage_template;
}

auto container_test(CargoUser &user, StorageTemplate &storage_template)
{
    auto container = user.create_container(RustString{"My Container"}, storage_template);
    std::cout << container.stringify().to_string() << std::endl;

    auto containers = user.get_containers();
    for (uint i = 0; i < containers.size(); ++i)
    {
        auto current_container = containers.at(i).unwrap();
        std::cout << container.stringify().to_string() << std::endl;
        user.delete_container(current_container);
        std::cout << "IN LOOP: " << current_container.stringify().to_string() << std::endl;
    }

    // this container is also mark deleted (deleted in loop)
    std::cout << "AFTER LOOP: " << container.stringify().to_string() << std::endl;
}

int main()
{
    CargoCfgProviderImpl cfg_provider{};
    CargoConfig cfg = collect_config(cfg_provider);
    // cfg.override_evs_url(RustString{"new url"});
    LocalSecureStorageImpl lss{};
    SharedMutexCargoLib cargo_lib;
    try
    {
        cargo_lib = create_cargo_lib(lss, cfg);
    }
    catch (const CargoLibCreationError_ErrorException &e)
    {
        std::cerr << e.reason().to_string() << std::endl;
        assert(false);
    }

    UserApi user_api = cargo_lib.user_api();

    try
    {
        MnemonicPayload mnemonic = user_api.generate_mnemonic();
        std::string mnemonic_str = mnemonic.stringify().to_string();
        std::cout << "Generated mnemonic: " << mnemonic_str << std::endl;

        RustVec<String> words_vec = mnemonic.get_vec();
        for (uint i = 0; i < words_vec.size(); i++)
        {
            std::cerr << words_vec.at(i).unwrap().to_string() << std::endl;
        }

        String device_name = String("My Mac");

        try
        {
            CargoUser new_user = user_api.create_user_from_mnemonic(mnemonic, device_name);
            std::cout << "User successfully created from mnemonic\n";

            try
            {
                auto storage_template = foundation_storage_test(new_user);

                RustVec<StorageTemplate> storage_templates = new_user.get_storage_templates();
                StorageTemplate first_st = storage_templates.at(0).unwrap();
                std::cout << first_st.stringify().to_string() << std::endl;

                container_test(new_user, storage_template);
            }
            catch (const RustExceptionBase &e)
            {
                std::cerr << e.reason().to_string() << std::endl;
            }

            try
            {
                CargoUser user = user_api.get_user();
                std::cout << "User: " << user.stringify().to_string() << std::endl;
            }
            catch (const RustExceptionBase &e)
            {
                std::cerr << e.reason().to_string() << std::endl;
                assert(false);
            }
        }
        catch (const RustExceptionBase &e)
        {
            std::cerr << e.reason().to_string() << std::endl;
            assert(false);
        }

        try
        {
            RustVec<u8> entropy;
            user_api.create_user_from_entropy(entropy, device_name); // Expected to fail
            assert(false);
        }
        catch (const RustExceptionBase &e)
        {
            std::cerr << e.reason().to_string() << std::endl;
        }
    }
    catch (const RustExceptionBase &e)
    {
        std::cerr << e.reason().to_string() << std::endl;
        assert(false);
    }

    config_parser_test();
}
