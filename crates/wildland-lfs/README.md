# Local Filesystem Backend

Exemplary, not intended for production use backend driver for Wildland DFS. It uses a local filesystem
directory as a Storage.

Created for testing purposes on Linux and apple machines - not tested for other platforms.

## Example template

```yaml
name: `Example template`
uuid: `00000000-0000-0000-0000-0000000000001`
backend_type: LFS
template:
    local_dir: `/home/user/storage/`
    container_prefix: `{{ CONTAINER_NAME }}`
```

`container_prefix` specifies a directory inside a storage location (`local_dir` param).