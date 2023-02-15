# Create user

All functionalities related to user are encapsulated within
[`UserApi`](https://docs.wildland.dev/docs/wildland/lld/doc/wildland_cargo_lib/api/user/struct.UserApi.html)
object, which can be obtained with the `user_api` method of `CargoLib`.

Using `UserApi` we are able to generate mnemonic (12-word phrase) and then pass it to the 
[`create_user_from_mnemonic`](https://docs.wildland.dev/docs/wildland/lld/doc/wildland_cargo_lib/api/user/struct.UserApi.html#method.create_user_from_mnemonic)
method along with device's name.

Sequence diagram of user creation process can be found [here](../architecture/user_creation.md).

```rust
let user_api = cargo_lib.user_api();
let mnemonic = user_api.generate_mnemonic().unwrap();
let user = user_api
    .create_user_from_mnemonic(&mnemonic, "device_name".to_string())
    .unwrap();
```