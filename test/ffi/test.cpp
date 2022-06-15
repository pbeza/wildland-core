#include <iostream>
extern "C" {
    #include "wildland.h"
    #include "SwiftBridgeCore.h"
}
#include "ffi.h"

int main()
{
    AdminManager admin_manager = create_admin_manager();

    ResultSeedPhrase seed_result = create_seed_phrase();
    if (seed_result.is_ok())
    {
        SeedPhrase seed_ok = seed_result.unwrap(); // it is safe to unwrap after `is_ok` check

        std::string seed_str = std::string(seed_ok.get_string().c_str());
        std::cout << "Generated seed: " << seed_str << std::endl;

        RustVec<String> words_vec = seed_ok.get_vec(); // String (starting with capital letter) is a rust type
        // TODO: implement iterator for RustVec ;)
        for (uint i = 0; i < words_vec.size(); i++)
        {
            std::cout << words_vec.at(i).c_str() << std::endl;
        }

        String name = String("Some generic name");
        ResultSharedMutexIdentity identity_result = admin_manager.create_master_identity_from_seed_phrase(name, seed_ok);
        std::cout << "Identity name: " << std::string(identity_result.unwrap().get_name().c_str()) << std::endl;

        OptionalSharedMutexIdentity optional_identity = admin_manager.get_master_identity(); // The same identity as inside the result above
        if (optional_identity.is_some())
        {
            SharedMutexIdentity identity = optional_identity.unwrap();

            std::cout << "Identity name: " << identity.get_name().c_str() << std::endl;
            String name = String("New name 3");
            identity.set_name(name); // Identity can be mutated
            std::cout << "Identity name: " << identity.get_name().c_str() << std::endl;
        }

        String email = String("test@email.com");
        admin_manager.set_email(email);
        ResultVoidType sending_result = admin_manager.request_verification_email(); // Code is hardcoded for now
        if (sending_result.is_ok())
        {
            String email_verification = String("123456");
            ResultVoidType verification_result = admin_manager.verify_email(email_verification);
            if (verification_result.is_ok())
            {
                std::cout << "Verification successful" << std::endl;
            }
            else
            {
                std::cout << verification_result.unwrap_err().to_string().c_str() << std::endl;
            }
        }
    }
    else
    {
        ErrorType seed_err = seed_result.unwrap_err();
        // error interface is extendable but for now it exposes methods for getting message and code
        std::string err_msg = std::string(seed_err.to_string().c_str());
        uint32_t code = seed_err.code();

        std::cout << "Error msg: " << err_msg << " \nError code: " << code << std::endl;
    }
}