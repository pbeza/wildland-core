#include <iostream>
#include "ffi_cxx.h"

int main()
{
    String lss_path = String("lss.yaml");
    CargoLib cargo_lib = create_cargo_lib(lss_path).unwrap();
    UserApi user_api = cargo_lib.user_api();

    ResultMnemonicPayload mnemonic_result = user_api.generate_mnemonic();
    if (mnemonic_result.is_ok())
    {
        MnemonicPayload mnemonic = mnemonic_result.unwrap(); // it is safe to unwrap after `is_ok` check

        std::string mnemonic_str = mnemonic.get_string().to_string();
        std::cout << "Generated mnemonic: " << mnemonic_str << std::endl;

        RustVec<String> words_vec = mnemonic.get_vec(); // String (starting with capital letter) is a rust type
        for (uint i = 0; i < words_vec.size(); i++)
        {
            std::cout << words_vec.at(i).to_string() << std::endl;
        }
        String device_name = String("My Mac");
        user_api.create_user_from_mnemonic(mnemonic, device_name).unwrap();
        std::cout << "User successfully created from mnemonic";
        UserPayload user = user_api.get_user().unwrap().unwrap();
        std::cout << "User: " << user.get_string().to_string() << std::endl;
    }
    else
    {
        ErrorType mnemonic_err = mnemonic_result.unwrap_err();
        // error interface is extendable but for now it exposes methods for getting message and code
        std::string err_msg = mnemonic_err.to_string().to_string();
        uint32_t code = mnemonic_err.code();

        std::cout << "Error msg: " << err_msg << " \nError code: " << code << std::endl;
    }
}