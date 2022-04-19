#!/usr/bin/env python3

import argparse

import wildland_cli_python_bindings

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    print(f"[+] Wildland CLI version {wildland_cli_python_bindings.get_python_cli_version()}")
    print(f'[+] Wildland Admin Manager version: {wildland_cli_python_bindings.get_admin_manager_version()}')
    print("[+] Core library version:")
    print(wildland_cli_python_bindings.get_corex_version_verbose())
