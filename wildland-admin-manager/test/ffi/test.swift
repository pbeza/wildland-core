print("TODO: Swift FFI Testsuite");

var seed_phrase_result = swift_create_seed_phrase();
if seed_phrase_result.is_ok() {
    print("Seed phrase result is OK")
}