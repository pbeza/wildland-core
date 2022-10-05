
// This test file is not supported since ffi-macro v.0.2.0

class CargoCfgProviderImpl: CargoCfgProvider {
    public override func getLogLevel() -> RustString {
        return RustString("info")
    }
    public override func getLogFile() -> OptionalString {
        return newNoneString()
    }

    public override func getEvsUrl() -> RustString {
        return RustString("http://localhost:5000/");
    }
    public override func getEvsRuntimeMode() -> RustString {
        return RustString("PROD");
    }
    public override func getEvsCredentialsPayload() -> RustString {
        return RustString(""); // irrelevant in case of evs prod mode
    }
}

class LocalSecureStorageImpl : LocalSecureStorage {
    private var store = [String : RustVec<u8>]()

    /// Inserts a key-value pair into the LSS.
    /// If the map did not have this key present, None is returned.
    /// If the map did have this key present, the value is updated, and the old value is returned.
    public override func insert(_ key: RustString,_ value: RustVec<u8>) -> LssOptionalBytesResult {
        let std_key = key.toString()
        let result = store[std_key] != nil
            ? newOkLssOptionalBytes(newSomeBytes(store[std_key]!))
            : newOkLssOptionalBytes(newNoneBytes())
        store[std_key] = value;
        return result;
        // return new_err_lss_optional_bytes(RustString("Err")); // EXAMPLE: returning error
    }

    /// Returns a copy of the value corresponding to the key.
    public override func get(_ key: RustString) -> LssOptionalBytesResult
    {
        let std_key = key.toString()
        return store[std_key] != nil
            ? newOkLssOptionalBytes(newSomeBytes(store[std_key]!))
            : newOkLssOptionalBytes(newNoneBytes())
    }

    /// Returns true if the map contains a value for the specified key.
    public override func containsKey(_ key: RustString) -> LssBoolResult
    {
        let std_key = key.toString()
        return newOkLssBool(store[std_key] != nil)
    }

    /// Returns all keys in arbitrary order.
    public override func keys() -> LssVecOfStringsResult
    {
        let keys = RustVec<RustString>(RustString.createNewRustVec())
        for (key, _) in store {
            keys.push(RustString(key))
        }
        return newOkLssVecOfStrings(keys)
    }

    /// Removes a key from the map, returning the value at the key if the key was previously in the map.
    public override func remove(_ key: RustString) -> LssOptionalBytesResult
    {
        let std_key = key.toString()
        var result: LssOptionalBytesResult
        if (store[std_key] != nil)
        {
            result = newOkLssOptionalBytes(newSomeBytes(store[std_key]!))
            store[std_key] = nil
        }
        else
        {
            result = newOkLssOptionalBytes(newNoneBytes())
        }
        return result
    }

    /// Returns the number of elements in the map.
    public override func len() -> LssUsizeResult
    {
        return newOkLssUsize((usize)(store.count))
    }

    /// Returns true if the map contains no elements, false otherwise.
    public override func isEmpty() -> LssBoolResult
    {
        return newOkLssBool(store.isEmpty)
    }
}

print("Swift FFI Test Suite")
do {
    let lss = LocalSecureStorageImpl()
    let cfg = collectConfig(CargoCfgProviderImpl())
    let cargo_lib = try createCargoLib(lss, cfg)
    let user_api = cargo_lib.userApi()
    let mnemonic = try user_api.generateMnemonic()
    print(mnemonic.getString().toString())
    let _ = try user_api.createUserFromMnemonic(mnemonic, RustString("My Mac"))
    print("User successfully created from mnemonic")
    let user = try user_api.getUser()
    print("User: " + user.getString().toString())

    do {
        let config_bytes: RustVec<u8> = RustVec(u8.createNewRustVec())
        let raw_config = "{\"log_level\": \"trace\"}"
        for ch in raw_config.utf8 {
            config_bytes.push(ch);
        }
        let parsed_cfg: CargoConfig = try parseConfig(config_bytes)
        let _ = try createCargoLib(lss, parsed_cfg)
    } catch let err as CargoLibCreationExc_FailureException {
        print(err.reason().toString())
    }
} catch let err as RustExceptionBase {
    print(err.reason().toString())
}
