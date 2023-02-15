# Release process

The purpose of this document is to describe the release process within Wildland
Core project.

## Branching

The branching strategy used in the Wildland Core project is the **gitflow**
strategy as per **RFC 0002**. The aforementioned strategy defined four sets of
branches:

- development and feature branches
- release/stabilization branches
- stable and production branches
- hotfix branches

### Ownership

The development branches are owned by each and every developer. Even the
`develop` branch is owned by the developers with the exception that every merge
into that branch must go through a code review by peers.

The release branches are created and merged into master by one of the project
maintainers. Maintainers (or, release masters) should agree amongst each other
who is responsible to deliver a particular release.

Stable and production branches have no owners as they are managed by the
automated CD workflows. Whoever designs or implements such workflow may be
considered an owner of those branches.

### Release branches names

Every release branch must have the following format:

```
release/v0.0.0
```

where `0.0.0` is `MAJOR.MINOR.PATCH` semantic versioning format.

### Protected branches

`master` and `release/*` are protected branches in the Wildland Core repository.

## Tagging

Neither developers nor maintainers should push any tags to the remote. Tagging
should be managed by the CI automation.

Currently we only use **release tags** that have the following format

```
v0.0.0
```

where `0.0.0` is `MAJOR.MINOR.PATCH` semantic versioning format.

Those tags are created during CD flow, right after cargo crates are published in
the registry. The tool that creates those tags is called [cargo-release]
(https://github.com/crate-ci/cargo-release/) and is executed as one of the CI
jobs that are run on `master` branch.

### Protected tags

`v*`, `*-release*` and `*-rc*` are protected tags in the Wildland Core
repository.

## Crates versions bumping

There must be no manual crates version bumping with the Wildland Core workspace.
Crates versions from now on are bumped only by the project maintainers in
release branches as well as by CD workflow on master branch upon crates
publishing.

## Release flow

Since we follow the gitflow strategy, this section will only extend the
non-standard parts.

### Setup

Once release is planned, and the scope is defined by project management, a
maintainer will create a release branch, branching of `develop`. Afterwards the
maintainer must bump the crates version to the pre-release format, namely

```
v0.0.0-rc.0
```

Maintainers should use helper scripts to prevent mistakes that may occur during
manual actions.

### Release cycle

Once the branch is created, the project enters release mode. During that time
there should be no features merged into develop whilst bugfixes, documentation
updates etc. should go straight to the active release branch.

Now is the time for the QA team to test the release as well as the maintainer
should gather release notes, update changelog, etc. Once the release is ready
the maintainer merges release branch into master, triggering the CD workflow.
Lastly the `develop` branch is rebased on `master` and the regular development
flow continues.

Note that the release branch lifespan may be very short, i.e. it may as well be
created and merged into master during a single day)
