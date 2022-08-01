import unittest
import wildland


class TestCargoLib(unittest.TestCase):
    def setUp(self):
        self.cargo_lib = wildland.create_cargo_lib(
            wildland.RustString("lss.yaml")).unwrap()

    def test_mnemonic_generation(self):
        mnemonic_result = self.cargo_lib.user_api().generate_mnemonic()
        mnemonic_result.is_ok()
        mnemonic = mnemonic_result.unwrap()
        print(mnemonic.get_string().to_string())

    def test_create_user_from_mnemonic(self):
        user_api = self.cargo_lib.user_api()
        mnemonic = user_api.generate_mnemonic().unwrap()
        user_api.create_user_from_mnemonic(mnemonic, wildland.RustString("My Mac")).unwrap()
        print("User successfully created from mnemonic")

if __name__ == '__main__':
    unittest.main()
