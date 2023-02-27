
// This test file is not supported since ffi-macro v.0.2.0

class CargoCfgProviderImpl: CargoCfgProvider {
    public override func getUseLogger() -> bool {
        return true
    }
    public override func getLogLevel() -> RustString {
        return RustString("info")
    }
    public override func getLogUseAnsi() -> bool {
        return false
    }
    public override func getLogFileEnabled() -> bool {
        return false
    }
    public override func getLogFilePath() -> RustOptional<RustString> {
        return Optional.none.toRustOptional()
    }
    public override func getLogFileRotateDirectory() -> RustOptional<RustString> {
        return Optional.none.toRustOptional()
    }
    public override func getFoundationCloudEnvMode() -> FoundationCloudMode {
        return FoundationCloudMode_Dev
    }
    public override func getRedisUrl() -> RustString {
        return RustString("redis://127.0.0.1/0")
    }
}

class LocalSecureStorageImpl : LocalSecureStorage {
    private var store = [String : RustString]()

    /// Inserts a key-value pair into the LSS.
    /// If the map did not have this key present, None is returned.
    /// If the map did have this key present, the value is updated, and the old value is returned.
    public override func insert(_ key: RustString,_ value: RustString) -> OptionalRustStringResultWithLssError {
        let std_key = key.toString()
        let result = store[std_key] != nil
            ? OptionalRustStringResultWithLssError.from_ok(Optional.some(store[std_key]!).toRustOptional())
            : OptionalRustStringResultWithLssError.from_ok(Optional.none.toRustOptional())
        store[std_key] = value;
        return result;
        // return new_err_lss_optional_bytes(RustString("Err")); // EXAMPLE: returning error
    }

    /// Returns a copy of the value corresponding to the key.
    public override func get(_ key: RustString) -> OptionalRustStringResultWithLssError
    {
        let std_key = key.toString()
        return store[std_key] != nil
            ? OptionalRustStringResultWithLssError.from_ok(Optional.some(store[std_key]!).toRustOptional())
            : OptionalRustStringResultWithLssError.from_ok(Optional.none.toRustOptional())
    }

    /// Returns true if the map contains a value for the specified key.
    public override func containsKey(_ key: RustString) -> boolResultWithLssError
    {
        let std_key = key.toString()
        return boolResultWithLssError.from_ok(store[std_key] != nil)
    }

    /// Returns all keys in arbitrary order.
    public override func keys() -> VecRustStringResultWithLssError
    {
        let keys = RustVec<RustString>(RustString.createNewRustVec())
        for (key, _) in store {
            keys.push(RustString(key))
        }
        return VecRustStringResultWithLssError.from_ok(keys)
    }

    /// Returns all keys in arbitrary order.
    public override func keysStartingWith(_ prefix: RustString) -> VecRustStringResultWithLssError
    {
        let keys = RustVec<RustString>(RustString.createNewRustVec())
        let std_prefix = prefix.toString()
        for (key, _) in store {
            if (key.starts(with: std_prefix)) {
                keys.push(RustString(key))
            }
        }
        return VecRustStringResultWithLssError.from_ok(keys)
    }

    /// Removes a key from the map, returning the value at the key if the key was previously in the map.
    public override func remove(_ key: RustString) -> OptionalRustStringResultWithLssError
    {
        let std_key = key.toString()
        var result: OptionalRustStringResultWithLssError
        if (store[std_key] != nil)
        {
            result = OptionalRustStringResultWithLssError.from_ok(Optional.some(store[std_key]!).toRustOptional())
            store[std_key] = nil
        }
        else
        {
            result = OptionalRustStringResultWithLssError.from_ok(Optional.none.toRustOptional())
        }
        return result
    }

    /// Returns the number of elements in the map.
    public override func len() -> usizeResultWithLssError
    {
        return usizeResultWithLssError.from_ok((usize)(store.count))
    }

    /// Returns true if the map contains no elements, false otherwise.
    public override func isEmpty() -> boolResultWithLssError
    {
        return boolResultWithLssError.from_ok(store.isEmpty)
    }
}

print("Swift FFI Test Suite")
do {
    let lss = LocalSecureStorageImpl()
    let cfg = try collectConfig(CargoCfgProviderImpl())
    // CargoLib expects to get references with static lifetime so it is important not to inline
    // objects (e.g. LSS) initialization along with createCargoLib call
    // DO NOT: createCargoLib(LocalSecureStorageImpl(), CargoCfgProviderImpl())
    let cargo_lib = createCargoLib(lss, cfg)
    let user_api = cargo_lib.userApi()

    // Mnemonic can be restored or generated randomly
    let mnemonic_vec = RustVec<RustString>(RustString.createNewRustVec())
    let words = ["update", "inherit", "giant", "spray", "expire", "enforce", "animal", "ship", "congress", "weather", "camp", "endless"]
    for w in words {
        mnemonic_vec.push(RustString(w))
    }
    let restored_mnemonic = try user_api.createMnemonicFromVec(mnemonic_vec)

    let mnemonic = try user_api.generateMnemonic()
    print(mnemonic.stringify().toString())
    let new_user = try user_api.createUserFromMnemonic(mnemonic, RustString("My Mac"))
    print("User successfully created from mnemonic")
    let user = try user_api.getUser()
    print("User: " + user.stringify().toString())

    let _ = try user.findContainers(
        Optional.some( // Passing some filter turns on filtering
            orFilter(
                hasPathStartingWith(RustString("/some_path")),
                hasPathStartingWith(RustString("/other_path"))
            )
        ).toRustOptional(),
        MountState_MountedOrUnmounted)

    do {
        let config_bytes: RustVec<u8> = RustVec<u8>()
        let raw_config = "{\"log_level\": \"trace\", \"redis_connection_string\": \"redis://127.0.0.1/0\"}"
        for ch in raw_config.utf8 {
            config_bytes.push(ch);
        }
        let parsed_cfg: CargoConfig = try parseConfig(config_bytes)
        let cargo_lib = createCargoLib(lss, parsed_cfg)
    } catch let err as RustExceptionBase {
        print(err.reason().toString())
    }
} catch let err as RustExceptionBase {
    print(err.reason().toString())
}
