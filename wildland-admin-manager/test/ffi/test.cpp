#include <iostream>
#include "mod.rs.h"

using namespace wildland;
using namespace ::rust;

int main()
{
    Box<AdminManager> admin_manager = create_admin_manager();

    Box<SeedPhraseResult> seed_result = create_seed_phrase();
    if (seed_result->is_ok())
    {
        Box<SeedPhrase> seed_ok = seed_result->boxed_unwrap(); // it is safe to unwrap after `is_ok` check

        std::string seed_str = std::string(seed_ok->get_string());
        std::cout << "Generated seed: " << seed_str << std::endl;

        Vec<String> words_vec = seed_ok->get_vec(); // String (starting with capital letter) is a rust type
        for (String &word : words_vec)              // rust Vec can be used similarly as the std::vector (Opaque types inside vector are not supported)
        {
            std::cout << std::string(word) << std::endl;
        }

        Box<IdentityResult> identity_result = admin_manager->create_master_identity_from_seed_phrase(String{"Some generic name"}, seed_ok);
        std::cout << "Identity name: " << std::string(identity_result->boxed_unwrap()->get_name()) << std::endl;

        Box<OptionalIdentity> optional_identity = admin_manager->get_master_identity(); // The same identity as inside the result above
        if (optional_identity->is_some())
        {
            Box<DynIdentity> identity = optional_identity->boxed_unwrap();

            std::cout << "Identity name: " << std::string(identity->get_name()) << std::endl;
            identity->set_name(::rust::String{"New name 3"}); // Identity can be mutated
            std::cout << "Identity name: " << std::string(identity->get_name()) << std::endl;
        }

        admin_manager->set_email(::rust::String("test@email.com"));
        Box<EmptyResult> sending_result = admin_manager->request_verification_email(); // Code is hardcoded for now
        if (sending_result->is_ok())
        {
            Box<EmptyResult> verification_result = admin_manager->verify_email(::rust::String("123456"));
            if (verification_result->is_ok())
            {
                std::cout << "Verification successful" << std::endl;
            }
            else
            {
                std::cout << verification_result->boxed_unwrap_err()->to_string().c_str() << std::endl;
            }
        }
    }
    else
    {
        Box<AdminManagerError> seed_err = seed_result->boxed_unwrap_err();
        // error interface is extendable but for now it exposes methods for getting message and code
        std::string err_msg = std::string(seed_err->to_string());
        uint32_t code = seed_err->code();

        std::cout << "Error msg: " << err_msg << " \nError code: " << code << std::endl;
    }
}