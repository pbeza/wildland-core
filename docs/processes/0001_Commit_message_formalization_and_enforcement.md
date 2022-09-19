# Commit message formalization and enforcement

````text
Acceptance in progress
````

## Commit message formalization

In the repository, all issues titles, pull request titles and commit messages
must have prefixed with a tag.

Currently, we allow tags:

* `chore` - dependency updates, ci updates, refactorings,other changes that
  don't affect the project functionality
* `test` - adding or removing tests; it does not include changes to deployment
  or CI. These go under `chore.`
* `fix` - fixing a bug in **code**
* `feat` - adding new feature to the project
* `docs` - adding documentation (readmes, mdbook, plantuml etc.)

On merge, we **squash** commits and use one of 5 tags. We are not using the
scope, as it's basically useless - most of our changes affect everything. The
summary should describe which components were changed.

As for the long commit message:

```text
[WILX-0012] tag: summary

* tag: summary
* tag: summary
```

The parent tag is the main goal of the work, and the list contains all things
we did during implementation. E.g.:

```text
[WILX-0123] feat: add lazers

* chore: add deployment target on the moon
* docs: document usage of lazers (and how to keep them away from cats)
```

In case the commit is not connected to any task in jira, key [WILX-0000] is
an acceptable placeholder/replacement.

## Commit message enforcement

Commit message format is enforced via one of CI Actions/Workflows
