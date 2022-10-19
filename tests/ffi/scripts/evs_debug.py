import sys
import requests
import json


def main() -> int:
    email = "test@email.com"
    debug_credentials = {
        "id": "21f527a0-5909-4b00-9494-2de8cfb6ace1",
        "credentialID": "7b20c5c2fa565ee9797d58f788169630d57c36ec8d618456728be7353c943ee8",
        "credentialSecret": "ff5ea13d0e881aa1a1e909a37bf02073934eacbda663508613910e1d86ecd406"
    }
    port = "5000"
    host = f"http://localhost:{port}"

    debug_provision_res = requests.put(
        f"{host}/debug_provision?email={email}&credentials={json.dumps(debug_credentials)}")
    assert debug_provision_res.status_code == 200

    get_token_res = requests.get(
        f"{host}/debug_get_token?email={email}")
    assert get_token_res.status_code == 200

    print(get_token_res.text)


if __name__ == '__main__':
    sys.exit(main())
