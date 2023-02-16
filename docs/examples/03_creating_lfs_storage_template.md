# Create Storage Template for development purposes

This chapter covers how to create an LFS (Local Filesystem Storage) template.

More information about Storage Templates can be found [here](../features/storage_templates.md).

LFS Storage Template will generate [`StoragesManifests`](https://docs.wildland.dev/docs/wildland/lld/doc/wildland_corex/catlib_service/entities/trait.StorageManifest.html)
that point to a local filesystem directory. That means that this is not a full-fledged Storage in terms of Wildland features,
because it is not sharable between multiple user's devices. It is useful for development purposes though.

`StorageTemplate` can be deserialized from yaml or json formatted content.

```rust
let yaml_content = "
    template:
        local_dir: /home/users/wildland_data
        container_dir: '{{ CONTAINER_NAME }}'
    backend_type: LocalFilesystem
";

let mut lfs_template = StorageTemplate::from_yaml(yaml_content.as_bytes().to_vec()).unwrap();

lfs_template.set_name("random name".to_string()); // it is possible to give it a name
```

It is convenient to use just `{{ CONTAINER_NAME }}` as a `container_dir`. Then, DFS driver for LFS will expect the
existence of a directory named exactly as a created Container (container creation described [here](./04_create_and_mount_container.md))
in the location pointed by the `local_dir` value.

**DFS does not create a directory for a container** (it has to be created manually).

On other platforms, for deserializing a `StorageTemplate`, one can use `storage_template_from_json`
and `storage_template_from_yaml` global functions.