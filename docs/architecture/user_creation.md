# User Creation Process

## Cargo: User Creation flow

```mermaid
sequenceDiagram
app->>cargo: create user
cargo->>lss: asks for forest uuid
alt user exists
    cargo->>catlib: check forest uuid
    catlib->>cargo: uuid exists
    catlib->>catlib: error, user already exists
else user does not exist
    cargo->>catlib: check forest uuid
    catlib->>cargo: uuid does not exists
    cargo->>wl-crypto: new master identity
    cargo->>wl-crypto: new forest identity
    cargo->>wl-crypto: new device identity
    cargo->>lss: save forest uuid as default
    cargo->>lss: save forest identity
    cargo->>lss: save device identity
    cargo->>app: new user
end
```

User creation depends on multiple components, namely:

* `Cargo`
  * that facilitates high-level handling of user identities,
  * makes sure there is only one master identity and there are no collisions
* `Corex` components:
  * `LSS` - abstractions handling secure storage on specific platforms,
    stores private keys and identities
  * `wl-crypto` - handling the cryptographic operations
    creates the identities and keys required by other components
  * `catlib` - backend responsible for distributed storage, stores the public
    identities and configurations that can be shared between the devices
