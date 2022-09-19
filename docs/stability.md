# General Stability

Work In Progress Page!

## Glossary

Stable API - breaking change requires MAJOR version bump,
Experimental API - breaking change does not requires MAJOR

## Motivation

After release process is introduced, it is very likely that some parts of its
API may change over time and will break it or be incompatible.
To prevent bumping MAJOR version every release (or slowing down development by
stabilizing every feature) team decided to pick API parts that are mature and
should not change very often and stabilize it. The rest is considered
unstable/experimental - you can play with it but it might be prone to errors or
major changes.

This document describes state after release 1.1.0

## Legend

- ⛔ - Experimental API
- ⚠ - Partially stable API - used only for component
- ✅ - Stable API

## Stability

components and their features states:

### Component A ⚠

- Feature xyz_1 ✅
- Feature xyz 2 ⛔
