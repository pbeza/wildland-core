# User Creation Process

## Cargo: User Creation flow

```mermaid
sequenceDiagram
app->>cargo: create new identity
cargo->>lss: asks for forest
alt user exists
    cargo->>corex: check user
    corex->>cargo: user exists error
else user does not exist
    cargo->>corex: check user
    corex->>cargo: user does not exists
    cargo->>corex: create new identity
    corex->>cargo: new user
end
```

## Corex: Simplified User Creation

```mermaid
sequenceDiagram
    cargo->>catlib: get forest
    catlib->>cargo: error, not found
    cargo->>wl-crypto: new identity
    wl-crypto->>cargo: identity
    cargo->>lss: save identity
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

## Corex: Simplified Flow for Identity Generation

Please mind that most of the identities saved in catlib are composed only from
public keys. Security information, like private keys, are saved in the LSS

```mermaid
sequenceDiagram
    crypto->>crypto: derive identity (crypto identity)
    crypto->>crypto: derive master identity from crypto identity
    crypto->>crypto: derive device identity from master identity
    crypto->>catlib: create forest
    crypto->>catlib: save forest identity as default
    crypto->>catlib: save device identity
    crypto->>catlib: save user metadata
```
