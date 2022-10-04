# General Versioning

## Versioning scheme

- Project is versioned with [semver](https://semver.org/) (MAJOR.MINOR.PATCH)
- Major releases may break your product. Look for changelog/release notes to
  see if you need to update your code in order for it to work properly.
- Minor releases will add new features without breaking any functionality.
- Patch releases should be treated as hotfixes. They will not require any
- changes to your deployment configuration other than a version number change.
  They may introduce new config settings which will be optional.
- Major and minor releases might require changes to deployment configuration.

## Release

- On each release, repository state will be marked with a tag and artifacts
  will be built for all supported targets and platforms.
- Standard(non-tagged) releases will be made only from main branch.
- Tagged releases may be created from different branch, unless stated otherwise
  we do not guarantee theirs stability.

## Breaking changes

By breaking change, we mean a change in public api, that is not
backward-compatible. Public api defines how clients integrating with the
libraries can call use those, and what are the requirements for those.
Due to the project being specific, the internal API between the components is
also considered public. To learn more, please review chapter below for reference.

## Internal components changes

Each release will have a fixed version matrix i.e:

- component A : 1.0
- component B : 1.1
- component C : 0.2

Tis will be recommended and stabilised versions of compoenents for the release.
Release means that components are confirmed to work together and were
extensively tested. However, usage of different versioning is possible taking
above into considerations: some things can, but not must, be broken or
incompatible. In that case, tT the very least, versioning matrix should be taken
into consideration as a minimal supported versions of components.

## Release schedule

There is no strict release schedule at this point. New versions will be
released when there is a need for them(e.g. new feature implemented).
