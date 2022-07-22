import unittest
import wildland


class TestAdminManager(unittest.TestCase):
    def setUp(self):
        self.admin_manager = wildland.create_admin_manager(wildland.RustString("/tmp/lss.yaml"))

    def test_mnemonic_generation(self):
        mnemonic_result = self.admin_manager.user_api().generate_mnemonic()
        mnemonic_result.is_ok()
        mnemonic = mnemonic_result.unwrap()
        print(mnemonic.get_string().to_string())


if __name__ == '__main__':
    unittest.main()
