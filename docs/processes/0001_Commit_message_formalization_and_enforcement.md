# Commit message formalization and enforcement

## Commit message formalization

In commit message, the line length must be limited to 80 characters per line.
This applies to all lines in the commit, including, but not limited to the
list of fixes, features, and commit message itself.

In the repository, all issues titles, pull request titles and commit messages
must be prefixed with a tag.

Currently, following tags are allowed:

* `chore` - dependency updates, CI updates, refactorings, other changes that
  don't affect the project functionality
* `test` - adding or removing tests; it does not include changes to deployment
  or CI. These go under `chore.`
* `fix` - fixing a bug in **code**
* `feat` - adding new feature to the project
* `docs` - adding documentation (readmes, mdbook, plantuml etc.)

On merge, commits must be **squashed** and one of 5 tags must be used. Scope in
the project does not matter (i.e. COMPONENT_XYZ) - most of the changes affect
multiple components. The summary should describe which components were changed.

As for the long commit message:

```text
tag: [TASK] summary

* tag: summary
* tag: summary
```

The parent tag is the main goal of the work, and the list contains all things
we did during implementation. E.g.:

```text
feat: [WILX-0123] add lazers

* chore: add deployment target on the moon
* docs: document usage of lazers (and how to keep them away from cats)
```

In case the commit is not connected to any task in jira, the `[TASK]`
can be dropped.

```text
fix: stray cats

* fix: added code to handle stray cats
* docs: documented stray cats and their ability to shoot lazers
```


## Commit message enforcement

Commit message format is enforced via one of CI Actions/Workflows
