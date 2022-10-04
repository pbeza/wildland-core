#include <iostream>
#include <unordered_map>
#include <chrono>
#include <thread>
#include <memory>
#include "ffi_cxx.h"

class CargoCfgProviderImpl : public CargoCfgProvider
{
    String get_log_level() override
    {
        return RustString("info");
    }
    OptionalString get_log_file() override
    {
        return new_none_string();
    }
    String get_evs_url() override
    {
        return RustString("http://localhost:5000/");
    }
    String get_evs_runtime_mode() override
    {
        return String{"DEBUG"};
    }
    String get_evs_credentials_payload() override
    {
        return RustString{"{\"id\": \"21f527a0-5909-4b00-9494-2de8cfb6ace1\","
                          "\"credentialID\": \"7b20c5c2fa565ee9797d58f788169630d57c36ec8d618456728be7353c943ee8\","
                          "\"credentialSecret\": \"ff5ea13d0e881aa1a1e909a37bf02073934eacbda663508613910e1d86ecd406\"}"};
    }
};

class LocalSecureStorageImpl : public LocalSecureStorage
{
    /// Inserts a key-value pair into the LSS.
    /// If the map did not have this key present, None is returned.
    /// If the map did have this key present, the value is updated, and the old value is returned.
    LssOptionalBytesResult insert(RustString key, RustVec<u8> value) override
    {
        std::cout << "LSS insert C++ impl\n";
        auto std_key = key.to_string();
        LssOptionalBytesResult result;
        if (store.contains(std_key))
        {
            result = new_ok_lss_optional_bytes(new_some_bytes(store[std_key]));
        }
        else
        {
            result = new_ok_lss_optional_bytes(new_none_bytes());
        }
        store[std_key] = value;
        return result;
        // return new_err_lss_optional_bytes(String{"Err"}); // EXAMPLE: returning error
    }

    /// Returns a copy of the value corresponding to the key.
    LssOptionalBytesResult get(RustString key) override
    {
        std::cout << "LSS get C++ impl\n";
        auto std_key = key.to_string();
        if (store.contains(std_key))
        {
            return new_ok_lss_optional_bytes(new_some_bytes(store[std_key]));
        }
        else
        {
            return new_ok_lss_optional_bytes(new_none_bytes());
        }
    }

    /// Returns true if the map contains a value for the specified key.
    LssBoolResult contains_key(RustString key) override
    {
        std::cout << "LSS contains_key C++ impl\n";
        auto std_key = key.to_string();
        return new_ok_lss_bool(store.contains(std_key));
    }

    /// Returns all keys in arbitrary order.
    LssVecOfStringsResult keys() override
    {
        std::cout << "LSS keys C++ impl\n";
        RustVec<RustString> keys;
        for (const auto &[k, v] : store)
        {
            keys.push(RustString{k});
        }
        return new_ok_lss_vec_of_strings(keys);
    }

    /// Removes a key from the map, returning the value at the key if the key was previously in the map.
    LssOptionalBytesResult remove(RustString key) override
    {
        std::cout << "LSS remove C++ impl\n";
        auto std_key = key.to_string();
        LssOptionalBytesResult result;
        if (store.contains(std_key))
        {
            result = new_ok_lss_optional_bytes(new_some_bytes(store[std_key]));
            store.erase(std_key);
        }
        else
        {
            result = new_ok_lss_optional_bytes(new_none_bytes());
        }
        return result;
    }

    /// Returns the number of elements in the map.
    LssUsizeResult len() override
    {
        std::cout << "LSS len C++ impl\n";
        return new_ok_lss_usize(store.size());
    }

    /// Returns true if the map contains no elements, false otherwise.
    LssBoolResult is_empty() override
    {
        std::cout << "LSS is_empty C++ impl\n";
        return new_ok_lss_bool(store.empty());
    }

private:
    std::unordered_map<std::string, RustVec<u8>> store = {};
};

void config_parser_test() // test
{
    RustVec<u8> config_bytes{};
    std::string raw_config = "{\"log_level\": \"trace\", \"evs_runtime_mode\": \"PROD\", \"evs_url\": \"http://some_evs_endpoint/\"}";
    for (const auto ch : raw_config)
    {
        config_bytes.push(ch);
    }
    LocalSecureStorageImpl lss{};
    CargoConfig cargo_cfg = parse_config(config_bytes);
    try
    {
        CargoLib cargo_lib = create_cargo_lib(lss, cargo_cfg);
    }
    catch (const CargoLibCreationExc_FailureException &e)
    {
        std::cout << e.reason().to_string() << std::endl;
    }
}

void foundation_storage_test(CargoLib &cargo_lib)
{
    try
    {
        FoundationStorageApi fsa_api = cargo_lib.foundation_storage_api();
        auto process_handle = fsa_api.request_free_tier_storage("test@email.com");
        auto verification_token = RustString{"in debug version this token is replaced for the one retrieved from evs debug endpoint"};
        fsa_api.verify_email(process_handle, verification_token);
    }
    catch (const RustExceptionBase &e)
    {
        std::cerr << e.reason().to_string() << std::endl;
    }
}

int main()
{
    CargoCfgProviderImpl cfg_provider{};
    CargoConfig cfg = collect_config(cfg_provider);
    LocalSecureStorageImpl lss{};
    CargoLib cargo_lib;
    try
    {
        cargo_lib = create_cargo_lib(lss, cfg);
    }
    catch (const CargoLibCreationExc_FailureException &e)
    {
        std::cerr << e.reason().to_string() << std::endl;
    }

    foundation_storage_test(cargo_lib);

    UserApi user_api = cargo_lib.user_api();

    std::this_thread::sleep_for(std::chrono::milliseconds(2000));
    try
    {
        MnemonicPayload mnemonic = user_api.generate_mnemonic(); // TODO WILX-220 MEMLEAK
        std::string mnemonic_str = mnemonic.get_string().to_string();
        std::cout << "Generated mnemonic: " << mnemonic_str << std::endl;

        RustVec<String> words_vec = mnemonic.get_vec();
        for (uint i = 0; i < words_vec.size(); i++)
        {
            std::cerr << words_vec.at(i).unwrap().to_string() << std::endl;
        }

        String device_name = String("My Mac");

        try
        {
            user_api.create_user_from_mnemonic(mnemonic, device_name); // TODO WILX-220 MEMLEAK
            std::cout << "User successfully created from mnemonic\n";

            try
            {
                UserPayload user = user_api.get_user();
                std::cout << "User: " << user.get_string().to_string() << std::endl;
            }
            catch (const UserRetrievalExc_NotFoundException &e)
            {
                std::cerr << e.reason().to_string() << std::endl;
            }
            catch (const UserRetrievalExc_UnexpectedException &e)
            {
                std::cerr << e.reason().to_string() << std::endl;
            }
        }
        catch (const UserCreationExc_FailureException &e)
        {
            std::cerr << e.reason().to_string() << std::endl;
        }

        try
        {
            RustVec<u8> entropy;
            user_api.create_user_from_entropy(entropy, device_name);
        }
        catch (const UserCreationExc_FailureException &e)
        {
            std::cerr << e.reason().to_string() << std::endl;
        }
    }
    catch (const MnemonicCreationExc_FailureException &e)
    {
        std::cerr << e.reason().to_string() << std::endl;
    }

    config_parser_test();
}