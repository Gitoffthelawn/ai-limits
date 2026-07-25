# Principles

## Build Principles

- Start release builds deliberately.
- Build each platform in a suitable native environment.
- Keep platform work explicit and publish only after all required builds pass.
- Keep signing and notarization credentials outside the repository.

## Artifact Principles

- Produce a downloadable artifact for every supported platform.
- Artifact names should be stable and human-readable.
- Preserve platform-specific metadata while packaging and verify the distributable artifact.
- Keep transient build storage separate from the release distribution channel.

## Release Principles

- Keep versioning rules in [versioning](versioning.md).
- A pre-release may be useful and downloadable while still being incomplete and bug-prone.
- Publish separate release assets per operating system so users download only what they need.
- Do not present unstable releases as stable or store-ready.
- Do not present a macOS release as notarized unless notarization and stapling completed.
