import ctypes
import wildland


print("Python FFI Testsuite")

# Python specific pointers deref function:


def deref_ptr(ptr, typ):
    return ctypes.cast(int(ptr), ctypes.POINTER(typ))[0]


admin_manager = wildland.create_admin_manager()
seed_result = wildland.create_seed_phrase()
if seed_result.is_ok():
    seed = seed_result.unwrap()
    identity_result = admin_manager.create_master_identity_from_seed_phrase(
        wildland.RustString("name 1"), seed)
    identity = identity_result.unwrap()
    print(identity.get_name().c_str())
    identity.set_name(wildland.RustString("name 2"))
    print(identity.get_name().c_str())

    identity_opt = admin_manager.get_master_identity()
    if identity_opt.is_some():
        identity_second_ref = identity_opt.unwrap()
        identity_second_ref.set_name(wildland.RustString("name 3"))
        print(identity_second_ref.get_name().c_str())

    print(identity.get_name().c_str())

    admin_manager.set_email(wildland.RustString("test@email.com"))
    sending_result = admin_manager.send_verification_code()  # Code is hardcoded for now
    if sending_result.is_ok():
        verification_result = admin_manager.verify_email(
            wildland.RustString("123456"))
        if verification_result.is_ok():
            print("Verification successfull")
        else:
            print(verification_result.unwrap_err().to_string().c_str())
