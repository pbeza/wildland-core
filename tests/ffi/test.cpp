#include <iostream>
extern "C" {
    #include "ffi_swift.h"
    #include "SwiftBridgeCore.h"
}
#include "ffi_cxx.h"

int main()
{
    String lss_path = String("lss.yaml");
    AdminManager admin_manager = create_admin_manager(lss_path).unwrap();

    ResultSeedPhrase mnemonic_result = admin_manager.user_api().generate_mnemonic();
    if (mnemonic_result.is_ok())
    {
        SeedPhrase mnemonic_ok = mnemonic_result.unwrap(); // it is safe to unwrap after `is_ok` check

        std::string mnemonic_str = mnemonic_ok.get_string().to_string();
        std::cout << "Generated mnemonic: " << mnemonic_str << std::endl;

        RustVec<String> words_vec = mnemonic_ok.get_vec(); // String (starting with capital letter) is a rust type
        for (uint i = 0; i < words_vec.size(); i++)
        {
            std::cout << words_vec.at(i).to_string() << std::endl;
        }
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