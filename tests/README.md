# C++ Tests

## EVS communication test

There is a `foundation_storage_test` function in *test.cpp* which requires set up EVS server in order to retrieve new storage credentials. In case of absence of that server the test will pass because exception (caused by refused connection) is going to be caught (it means no problems for CI).
In order to run the process of storage credentials retrieval one must take a few steps:

1. Set up a server 

    It can be done by downloading branch `encryption-off` of EVS repository (https://gitlab.com/wildland/cargo/email-verification-service) and running docker container with `make run` in root directory.

    Server must be run in `DEV` and with token delivery set to `HTTP`. Instruction how to do that can be found in EVS README.md file.

    **IMPORTANT** Use `encryption-off` branch since encryption is not supported yet.

2. Run c++ test

    E.g. command executed in root directory of this repo:

    ```
    cargo build --features "bindings"  && \
        g++ -std=c++20 -w ./tests/ffi/test.cpp \
        -I ./crates/wildland-cargo-lib/_generated_ffi_code -L ./target/debug/ \
        -L ./crates/wildland-cargo-lib/_generated_ffi_code -l wildland_cargo_lib \
        -l dl -l pthread -lcrypto -lssl \
        -o ./test &&  ./test
    ```

    Test will stop on expected input (verification token)

3. Get verification token

    Run *tests/ffi/scripts/evs_debug.py*. I'll retrieve verification token from EVS server and will print it to the console.

    Both scripts uses the same hardcoded variables for now and the same configuration (e.g. evs port [default 5000]).

4. Paste token to the test input

This flow works only once because EVS server creates a record in its database for an email. In order to successfully run 3. step again, database volume of EVS container must be removed (default name of volume: `docker_db`).