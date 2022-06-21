import unittest
import wildland


class TestAdminManager(unittest.TestCase):
    def setUp(self):
        self.admin_manager = wildland.create_admin_manager()

    def test_seed_generation(self):
        seed_result = self.admin_manager.create_seed_phrase()
        seed_result.is_ok()
        seed = seed_result.unwrap()
        print(seed.get_string().c_str())

    def test_create_identities_from_seed(self):
        seed = self.admin_manager.create_seed_phrase().unwrap()
        identities_result = self.admin_manager.create_wildland_identities(
            seed, wildland.RustString("name 1"))
        identities_result.is_ok()
        return identities_result

    def test_identities(self):
        identities = self.test_create_identities_from_seed().unwrap()
        forest_id = identities.forest_id()
        device_id = identities.device_id()

        print(forest_id.to_string().c_str())
        print(forest_id.get_fingerprint_string().c_str())
        print(forest_id.get_private_key())

        print(device_id.get_name().c_str())
        assert device_id.get_name().c_str() == "name 1"
        device_id.set_name(wildland.RustString("name 2"))
        assert device_id.get_name().c_str() == "name 2"

        if device_id.save().is_ok():
            print("Device identity saved in a file")

    def test_successfully_verify_email(self):
        self.admin_manager.set_email(wildland.RustString("test@email.com"))
        sending_result = self.admin_manager.request_verification_email()
        assert sending_result.is_ok()
        assert self.admin_manager.verify_email(
            wildland.RustString("123456")).is_ok()


if __name__ == '__main__':
    unittest.main()
