# Wildland Admin Manager

This project was created by running:
```console
$ maturin init
```
and modifying the created files.

## How to run?

In order to build the crate and install it as a python module directly in the current virtualenv, run:
```console
$ python -m venv env
$ source env/bin/activate
$ pip install maturin pytest
$ maturin develop
$ python
>>> wildland_cli_python_bindings.get_admin_manager_version(
... )
'0.1.0'
>>> wildland_cli_python_bindings.get_admin_manager_version()
'0.1.0'
>>> wildland_cli_python_bindings.get_corex_version_verbose()
'[+] * CoreX version 0.1.0\n[+] * CatLib version 0.1.0\n[+] * Wallet version 0.1.1\n[+] * DFS version 0.1.0\n'
>>> wildland_cli_python_bindings.get_python_cli_version()
'0.1.0'
```

## What is what?

`maturin` is a tool for building and publishing Rust-based Python packages
