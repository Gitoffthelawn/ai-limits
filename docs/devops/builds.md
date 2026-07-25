# Desktop Builds

## Process

- A release build is started deliberately, not for every source-code change.
- Validate the intended version and release notes before allocating platform build resources.
- Build each supported platform in an appropriate native environment.
- Verify the distributable artifact, not only the build output.
- Publish only when every required platform artifact is available and valid.

The current implementation is [GitHub Actions](../../.github/workflows/desktop-build.yml). It is an implementation detail and may be replaced without changing this process.

## Current Security Policy

- macOS release artifacts must be signed, notarized, and stapled.
- Windows and Linux artifacts are currently unsigned; their status must be visible to users.
- Signing credentials must be held only by the protected build environment and never committed to the repository.

See [macOS signing](macos-signing.md) and [versioning](versioning.md).
