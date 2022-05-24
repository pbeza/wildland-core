#include <iostream>
#include "../../target/cxxbridge/wildland-admin-manager/src/ffi/mod.rs.h"

using namespace cargo::api;

int main()
{
    auto admin_manager = create_admin_manager();

    auto seed_result = create_seed_phrase();
    if (seed_result->is_ok())
    {
        auto &seed_ok_ref = seed_result->unwrap();
        auto seed_str = std::string(seed_ok_ref.get_string());
        std::cout << "Generated seed: " << seed_str << std::endl;

        auto identity_result = admin_manager->create_master_identity_from_seed_phrase(::rust::String{"Some generic name"}, seed_ok_ref);

        auto optional_identity = admin_manager->get_master_identity(); // The same identity as inside the result above
        if (optional_identity->is_some())
        {
            auto &identity = optional_identity->unwrap();

            std::cout << "Identity name: " << std::string(identity.get_name()) << std::endl;
            identity.set_name(::rust::String{"New name"});
            std::cout << "Identity name: " << std::string(identity.get_name()) << std::endl;
        }
    }
    else
    {
        auto &seed_err_ref = seed_result->unwrap_err();
        auto err_msg = std::string(seed_err_ref.to_string());
        auto code = seed_err_ref.code();
        std::cout << code << std::endl;
    }
}