# Desktop Releases

## Purpose

Provide collaborators and early users with one downloadable set of working desktop artifacts without asking them to build the application themselves. Current releases are explicitly unstable and are not store-ready.

## Release Rules

- A release is created only after all required platform artifacts pass their checks.
- The release version follows [versioning](versioning.md).
- The `Unreleased` section of [CHANGELOG.md](../../CHANGELOG.md) is the source of product-change text; commit messages are not a release-note source.
- Publish separate assets per operating system and state the security status of each platform.
- Publish one asset per installation method, so a platform never offers the same installation twice in different packaging. macOS publishes the disk image; the application bundle archive stays a build artifact.
- Do not present a macOS artifact as notarized unless the `full` signing mode completed.
- Stop publication if release metadata is invalid, the changelog is empty, or the source revision changed during the release process.

The current publication channel is a GitHub pre-release created by the [desktop workflow](../../.github/workflows/desktop-build.yml). The channel may change without changing these rules.
