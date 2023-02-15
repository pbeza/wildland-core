# Branching strategy

````text
=========================================================
shortened version is available at the end of the document
=========================================================
````

## Glossary

- Release Branch - A fork originating from the development branch, marking the
  point in time, in which a feature set is frozen, and an official release is
  made. example `1.3.1`
- Development Branch - The main, active branch to which all code that is
  currently worked on is merged. Only developer releases (a.k.a. release
  candidates) can be done from this main branch. example: `develop`
- `Develop` or `develop` - a common name used in this document to refer to
  development branch (see above)
- MAJOR.MINOR.PATCH - The standard notation for semantic versioning. The
  triplet represents three main categories of versions: major release,
  minor release, and patches. The importance of the version section decreases
  from left to right. Additionally, a bump in a specific category resets
  everything in categories to the right to 0.
- Release - A fixed build of either the release branch or development branch
  that is then published for public usage.

## Description

If we consider a release to be a collection of features added since a prior
release, our goal in producing releases, is to provide a stable set of
features that are accessible to end users, as well as maintaining a means by
which we can conveniently and correctly provide patches for said releases.

The branching strategy is a soft-restricted version of the gitflow.

When a release is planned, the next branch should be created (sprouted from the
common development branch) with name based on semversion (e.g. v1.3.12).

First, all the features should be completed and then merged into develop.
Develop should then be frozen and prepared for next release. A release branch
should be then created from the tip of the freeze (if any last-minute fixes are
applied).

If a feature is planned but not delivered in time before the freeze, then
afterward the freeze not delivered changes should not be committed
(only last-minute fixes to existing ones), until release branch is created,
main branch is treated as one and it can not get new features. Additionally, a
release branch's origin cannot be moved after creation and features cannot be
cherry-picked or merged afterward (no rebase or cherry-pick).

After the release branch is created (the stabilization branch), the
stabilization effort will proceed on that branch; however the development branch
can still be used for further development. That means, if the release deadline
will be missed by a developers, stabilization does not have to wait for the
feature development to release. That also means that no new features can't be
committed to the released branch, only stabilization patches and/or bug fixes
(which also have to be mirrored to development branch if applies)

If stabilization branch is deemed stable, it can be merged to master. Squashing
of the commits should not be required due to the fact that merges in the develop
branch are already tidy and in proper formatting. Example of such format:

```text
Feature WILX-000

- feat: cat lasers
- chore: more lasers
- fix: lazers do not hurt other cats anymore
- ...
```

After the merge, stabilization branch must be deleted.

### Versioning

Release branches are not supported long-term. The reason releases are happening
is to periodically provide user with stable and tested build artifacts, that are
used by non-rust clients - i.e. c++, c#, or swift clients.

#### Master Branch

Tags are REQUIRED on this branch.

Versions and tags on `master` branch must be in semver format:
For example `v1.3.1`

#### Development Branch

There are no tags on development branches.

#### Stabilization Branches

A branch must be created using the following format: `release/v0.0.0`

There are no tags on release branches.

This version is going to become next tag on master after merge.
This branch after merge must be deleted.

### Bugfixes

When bug fixes are required to fix incorrect behavior in a release. Code that
fixes the incorrect behavior must be first added to develop, then, when the team
decides its time introduce the fixes to the master branch, the code should be
merged using the standard procedure (as in release procedure). For the rules
regarding the release versioning, please check the `Versioning` chapter of this
document.

## TL;DR

This document is describing what is called, `gitflow`
and many materials are freely available online in different sources.
This document however introduces and specifies how tagging and release
preparations should look like. Besides that, everything should be 1:1 with
official documentation about the process.

### Patch Versioning

- Patches will result in a bump of the current version's PATCH number on the
  affected release branch.
- Patches will not result in bump in development (release candidates).
- Patches will be collected for each release. One patch does not have to result
  in one new release.

### Patch Application

- If the development branch is affected, the patch should be fixed and changes
  should be pushed to the development branch.
- If the supported release branch is also affected by the proposed patch...
  (to the developer's discretion):
  - ... and the fix is easily applicable, it should be cherry-picked from
    develop to release branch.
  - ... but the resulting fix is not easy to forward to the release branch
  (i.e. missing refactor, changes in history, conflicting features, etc), the
  fix has to be crafted and applied manually for each branch.

### Branch Tagging

- The master branch will be tagged by its (MAJOR.MINOR.PATCH) tag for each
  release
- No other branches should have their commits tagged
