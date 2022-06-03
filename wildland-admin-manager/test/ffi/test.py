import unittest
import wildland


def set_up_admin_manager_with_email_client(times_send: int):
    email_client_builder = wildland.create_email_client_mock_builder()
    email_client_builder.expect_send(wildland.RustString(
        "test@email.com"), wildland.RustString("123456"), times_send)
    email_client = email_client_builder.build()

    return wildland.create_admin_manager(email_client)


class TestAdminManager(unittest.TestCase):
    def setUp(self):
        self.admin_manager = set_up_admin_manager_with_email_client(0)

    def test_seed_generation(self):
        seed_result = wildland.create_seed_phrase()
        seed_result.is_ok()
        seed = seed_result.unwrap()
        print(seed.get_string().c_str())

    def test_create_identity_from_seed(self):
        seed = wildland.create_seed_phrase().unwrap()
        identity_result = self.admin_manager.create_master_identity_from_seed_phrase(
            wildland.RustString("name 1"), seed)
        identity_result.is_ok()

    def test_get_identity(self):
        self.test_create_identity_from_seed()
        identity_opt = self.admin_manager.get_master_identity()
        assert identity_opt.is_some()

    def test_identity_is_none(self):
        assert self.admin_manager.get_master_identity().is_some() == False


class TestEmailSending(unittest.TestCase):
    def setUp(self):
        self.admin_manager = set_up_admin_manager_with_email_client(1)

    def test_successfully_verify_email(self):
        self.admin_manager.set_email(wildland.RustString("test@email.com"))
        # Code is hardcoded for now
        sending_result = self.admin_manager.send_verification_code()
        assert sending_result.is_ok()
        assert self.admin_manager.verify_email(
            wildland.RustString("123456")).is_ok()

    def test_failed_email_verification(self):
        self.admin_manager.set_email(wildland.RustString("test@email.com"))
        # Code is hardcoded for now
        sending_result = self.admin_manager.send_verification_code()
        assert sending_result.is_ok()
        verification_err = self.admin_manager.verify_email(
            wildland.RustString("999999"))
        assert verification_err.is_ok() == False
        print(verification_err.unwrap_err().to_string().c_str())


class TestIdentity(unittest.TestCase):
    def setUp(self):
        self.admin_manager = set_up_admin_manager_with_email_client(0)
        seed_result = wildland.create_seed_phrase()
        self.identity_result = self.admin_manager.create_master_identity_from_seed_phrase(
            wildland.RustString("name 1"), seed_result.unwrap())

    def test_get_identity_name(self):
        assert self.identity_result.unwrap().get_name().c_str() == "name 1"

    def test_set_identity_name(self):
        self.identity_result.unwrap().set_name(wildland.RustString("name 2"))
        assert self.identity_result.unwrap().get_name().c_str() == "name 2"


if __name__ == '__main__':
    unittest.main()
