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
>>> import wildland_admin_manager_python
>>> wildland_admin_manager_python.get_version()
'0.1.0'
```

To run the pytest, activate venv and run:
```console
$ pytest src/wl_ver_printer/wl_ver_printer.py
```
