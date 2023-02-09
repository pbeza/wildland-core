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
    container_prefix: '{{ CONTAINER_NAME }}'
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
    container_prefix: 'Movies'
    user: 'Quentin Tarantino'
    password: 'secret password'
    url: http://storage.com
    paths:
        - path1
        - path2
```
