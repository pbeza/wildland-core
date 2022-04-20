import wildland_admin_manager_python

def get_wl_admin_manager_version() -> str:
    return wildland_admin_manager_python.get_version()

def test_version():
    assert get_wl_admin_manager_version() == '0.1.0'

if __name__ == '__main__':
    ver = get_wl_admin_manager_version()
    print(f'Wildland Admin Manager version: {ver}')
