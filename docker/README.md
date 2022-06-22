# Dockerfiles for Bindings building

### Development

For development (testing) purposes you may use the `docker-compose.yml` provided in this directory.
You shouldn't have to build the `wildland-bindings-builder` image as it's a dependency for the
actual testing image named `wildland-bindings`.

All docker tests are performed in linux environment.

In order to run a bindings test in an isolated enviroment, use the following command

```
docker-compose run wildland-bindings <target-language>
```

The list of target languages you'll find in `./scripts/entrypoint.bash` file.
