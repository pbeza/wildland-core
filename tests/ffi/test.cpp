#include <iostream>
#include "ffi_cxx.h"

int main()
{
    String lss_path = String("lss.yaml");
    try
    {
        CargoLib cargo_lib = create_cargo_lib(lss_path);
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
    catch (const CargoLibCreationExc_NotCreatedException &e)
    {
        std::cout << e.reason().to_string() << std::endl;
    }
}