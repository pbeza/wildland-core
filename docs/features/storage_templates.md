# Storage Template

Storage Templates provide some general information about storage location. Their only purpose is to be
filled with the container's parameters during container creation and to generate Storage Manifest 
(in opposition to a template it points to the storage location assigned to the particular container).

## Structure

Storage Templates have the following structure:

Fixed fields (independent of template type)
- **name** - optional name
- **uuid** - template uuid
- **backend_type** - indicates what Storage Backend should be used for Storage objects created out of a template
- **template** - type specific content; different Storages may require different data

## Supported Backend Types

**backend_type** matching is case-sensitive.

- `LocalFilesystem` - This is the backend used for developing purposes. It points to a local directory. Containers backed by such storage are not full-fledged containers in terms of the Wildland platform, cause they can not be shared for example.
To use this type of backend `lfs` feature must be turned on during compilation.

    Template-specific data, for this template, is:
    - `local_dir` - a path of the directory which includes subdirectories assigned to containers' storage
    - `container_dir` - a name of a subdirectory in the directory specified by the `local_dir` path. This subdirectory is exclusive container's storage.

        **IMPORTANT** DFS does not create this subdirectory.

- `S3` - S3 storage

    Template-specific data, for this template, is:
    - `access_key_id`
    - `secret_access_key`
    - `region`
    - `bucket_name`
    - `endpoint_url`

- `FoundationStorage` - this is the result of the onboarding process in Cargo. It should not be created manually.

    Not supported for now!


## Parameters placeholders

Storage templates uses [**Tera** library](https://tera.netlify.app/docs/) to fill templates with values being defined while container creation.

Supported parameters:

- **CONTAINER_NAME**
- **OWNER** - owner's pubkey as string
- **ACCESS_MODE** - `ReadWrite` | `ReadOnly`
- **CONTAINER_UUID**
- **PATHS** - set of container paths

## Format

Templates can be formatted with the following formats

- JSON
- YAML

## Examples

The following template:

```yaml
name: null
uuid: 00000000-0000-0000-0000-0000000000001     # template uuid
backend_type: ImaginaryStorage
template:
    container_dir: '{{ CONTAINER_NAME }}'
    user: '{{ OWNER }}'
    password: 'secret password'
    url: http://storage.com
    paths: '{{ PATHS }}'
```

after rendering it with the following parameters:

```
CONTAINER_NAME = "Movies"
OWNER = "Quentin Tarantino"     # in reality it would be a pubkey
ACCESS_MODE = ReadOnly
CONTAINER_UUID = 00000000-0000-0000-0000-0000000000002  # container uuid
PATHS = ["path1", "path2"]
```

would result in the following Storage:

```yaml
name: null
uuid: 00000000-0000-0000-0000-0000000000003 # Storage uuid
backend_type: ImaginaryStorage
data:
    container_dir: 'Movies'
    user: 'Quentin Tarantino'
    password: 'secret password'
    url: http://storage.com
    paths:
        - path1
        - path2
```
