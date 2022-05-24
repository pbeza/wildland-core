#include <iostream>
#include "../../target/cxxbridge/wildland-admin-manager/src/ffi/mod.rs.h"

using namespace cargo::api;

int main()
{
    auto admin_manager = create_admin_manager();

    auto seed_result = create_seed_phrase();
    if (seed_result->is_ok())
    {
        auto &seed_ok_ref = seed_result->unwrap(); // it is safe to unwrap after `is_ok` check

        auto seed_str = std::string(seed_ok_ref.get_string());
        std::cout << "Generated seed: " << seed_str << std::endl;

        auto words_vec = seed_ok_ref.get_vec();
        for (auto &word : words_vec) // rust Vec can be used similarly as the std::vector
        {
            std::cout << std::string(word) << std::endl;
        }

        auto identity_result = admin_manager->create_master_identity_from_seed_phrase(::rust::String{"Some generic name"}, seed_ok_ref);

        auto optional_identity = admin_manager->get_master_identity(); // The same identity as inside the result above
        if (optional_identity->is_some())
        {
            auto &identity = optional_identity->unwrap();

            std::cout << "Identity name: " << std::string(identity.get_name()) << std::endl;
            identity.set_name(::rust::String{"New name"}); // Identity can be mutated
            std::cout << "Identity name: " << std::string(identity.get_name()) << std::endl;
        }
    }
    else
    {
        auto &seed_err_ref = seed_result->unwrap_err();
        // error interface is extendable but for now it exposes methods for getting message and code
        auto err_msg = std::string(seed_err_ref.to_string());
        auto code = seed_err_ref.code();

        std::cout << "Error msg: " << err_msg << " \nError code: " << code << std::endl;
    }
}