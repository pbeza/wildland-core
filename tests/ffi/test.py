import unittest
import wildland


class TestCargoLib(unittest.TestCase):
    def setUp(self):
        self.cargo_lib = wildland.create_cargo_lib()

    def test_mnemonic_generation(self):
        mnemonic_result = self.cargo_lib.user_api().generate_mnemonic()
        mnemonic_result.is_ok()
        mnemonic = mnemonic_result.unwrap()
        print(mnemonic.get_string().to_string())


if __name__ == '__main__':
    unittest.main()
