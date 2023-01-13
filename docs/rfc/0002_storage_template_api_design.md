# StorageTemplate FFI API design

```
Title           : StorageTemplate FFI API design
Category        : Feature
Author(s)       : Szymon Bagiński <szymon@wildland.io>
Team            : Corex Team
Created         : 2023-01-13
Deadline        : 2023-01-20
Feature ID      : WILX-146
```

The Document presents StorageTemplate API proposals.

The following API, during implementation, should be extended with result/error types that are not relevant in terms of this document.

## Basic methods

```rust
// we can rename it, it is not named `to_string` so it won't be confused with rust `ToString` impl
fn stringify(self: &StorageTemplate) -> String;

// each template should define field `backend_type`, once template is created it shouldn't be changed
fn backend_type(self: &StorageTemplate) -> String;

// uuid usually is generated randomly or loaded from file
fn uuid(self: &StorageTemplate) -> Uuid; // uuid will be exposed from ffi as str or u128 probably

// name is optional field, it is only for user's convenient
fn name(self: &StorageTemplate) -> String;
fn set_name(self: &StorageTemplate);
```

## Parsers/Serializers

We could consider the following approaches:

1. every format has its static method

```rust
// methods `new_from_*` don't expect uuid and generate it randomly on the fly
fn new_from_json(content: Vec<u8>) -> StorageTemplate;
fn new_from_yaml(content: Vec<u8>) -> StorageTemplate;
// we don't have to support toml etc. but technically corex supports every format supported by serde extension crates which can reflect equivalent structures as json/yaml.
fn new_from_toml(content: Vec<u8>) -> StorageTemplate;

// it expects uuid field inside the content (e.g. template that has been already in use but it was dumped to a file)
fn load_from_json(content: Vec<u8>) -> StorageTemplate;
fn load_from_yaml(content: Vec<u8>) -> StorageTemplate;
fn load_from_toml(content: Vec<u8>) -> StorageTemplate;

fn to_json(self: &StorageTemplate) -> Vec<u8>;
fn to_yaml(self: &StorageTemplate) -> Vec<u8>;
fn to_toml(self: &StorageTemplate) -> Vec<u8>;
```

2. format specified by a parameter

```rust
enum TemplateFormat {
    Json,
    Yaml,
    Toml,
}

fn new_from(format: TemplateFormat, content: Vec<u8>) -> StorageTemplate;
fn load_from(format: TemplateFormat, content: Vec<u8>) -> StorageTemplate;
fn serialize(self: &StorageTemplate, format: TemplateFormat) -> Vec<u8>;
```

3. Try to parse different formats sequentially (not recommended)

```rust
// tries to parse e.g. json first, if it fails it tries further yaml etc.
fn new_from(content: Vec<u8>) -> StorageTemplate;
// the same as above
fn load_from(content: Vec<u8>) -> StorageTemplate;

// serializing as in one of the above solutions
```

## Template-specific data manipulation

We could expose (I don't know if it is useful) methods for manipulating template-specific data (each template
may define any set of fields within a `template` property).

```rust
fn get_property(key: String) -> String;
fn set_property(key: String, value: String);
```