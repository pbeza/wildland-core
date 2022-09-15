#include <iostream>
#include "ffi_cxx.h"

class LocalSecureStorageImpl : public LocalSecureStorage
{
    LssOptionalBytesResult insert(RustString key, RustVec<u8> value)
    {
        std::cout << "LSS insert C++ impl\n";
        return new_ok_lss_optional_bytes(new_some_bytes(RustVec<u8>{}));
        // return new_err_lss_optional_bytes(String{"Err"});
    }

    LssOptionalBytesResult get(RustString key)
    {
        std::cout << "LSS get C++ impl\n";
        return new_ok_lss_optional_bytes(new_some_bytes(RustVec<u8>{}));
        // return new_err_lss_optional_bytes(String{"Err"});
    }

    LssBoolResult contains_key(RustString key)
    {
        std::cout << "LSS contains_key C++ impl\n";
        return new_ok_lss_bool(true);
        // return new_err_lss_optional_bytes(String{"Err"});
    }

    LssVecOfStringsResult keys()
    {
        std::cout << "LSS keys C++ impl\n";
        return new_ok_lss_vec_of_strings(RustVec<String>{});
        // return new_err_lss_optional_bytes(String{"Err"});
    }

    LssOptionalBytesResult remove(RustString key)
    {
        std::cout << "LSS remove C++ impl\n";
        return new_ok_lss_optional_bytes(new_some_bytes(RustVec<u8>{}));
        // return new_err_lss_optional_bytes(String{"Err"});
    }

    LssUsizeResult len()
    {
        std::cout << "LSS len C++ impl\n";
        return new_ok_lss_usize(0);
        // return new_err_lss_optional_bytes(String{"Err"});
    }

    LssBoolResult is_empty()
    {
        std::cout << "LSS is_empty C++ impl\n";
        return new_ok_lss_bool(true);
        // return new_err_lss_optional_bytes(String{"Err"});
    }
};

int main()
{
    LocalSecureStorageImpl lss{};
    CargoLib cargo_lib = create_cargo_lib(lss);
    UserApi user_api = cargo_lib.user_api();

    try
    {
        MnemonicPayload mnemonic = user_api.generate_mnemonic(); // TODO WILX-220 MEMLEAK
        std::string mnemonic_str = mnemonic.get_string().to_string();
        std::cout << "Generated mnemonic: " << mnemonic_str << std::endl;

        RustVec<String> words_vec = mnemonic.get_vec(); // String (starting with capital letter) is a rust type
        for (uint i = 0; i < words_vec.size(); i++)
        {
            std::cout << words_vec.at(i).unwrap().to_string() << std::endl;
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
                std::cout << e.reason().to_string() << std::endl;
            }
            catch (const UserRetrievalExc_UnexpectedException &e)
            {
                std::cout << e.reason().to_string() << std::endl;
            }
        }
        catch (const UserCreationExc_NotCreatedException &e)
        {
            std::cout << e.reason().to_string() << std::endl;
        }

        try
        {
            RustVec<u8> entropy;
            user_api.create_user_from_entropy(entropy, device_name);
        }
        catch (const UserCreationExc_NotCreatedException &e)
        {
            std::cout << e.reason().to_string() << std::endl;
        }
    }
    catch (const MnemonicCreationExc_NotCreatedException &e)
    {
        std::cout << e.reason().to_string() << std::endl;
    }
}