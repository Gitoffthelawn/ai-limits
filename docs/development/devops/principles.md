# Principles

## GitHub Actions Principles

- Start with manual workflows.
- Use native runners per platform.
- Keep platform builds explicit and publish only after all required platform jobs pass.
- Do not require DMG for the first macOS GitHub Actions success.
- Keep signing and notarization secrets limited to the explicit macOS GitHub Actions path.
- Keep platform-specific commands explicit if a matrix makes the workflow hard to read.

## Artifact Principles

- GitHub Actions produces downloadable artifacts for macOS, Windows, and Linux.
- macOS GitHub Actions supports Apple Developer ID signing.
- Artifact names should be stable and human-readable.
- Artifact paths must remain based on actual GitHub Actions output, not assumptions.
- macOS `.app` is archived before upload so the bundle structure is preserved.
- GitHub Actions artifact retention is currently 14 days.
- Unstable desktop builds are published through GitHub pre-releases for easier collaborator access.
- Long-term stable release artifacts should be handled through stable GitHub Releases, not `/private/tmp`.

## Release Principles

- Keep versioning rules in [Versioning](versioning.md).
- Use GitHub's `pre-release` flag for current unstable desktop releases.
- Keep release titles short and avoid repeating the repository name or full tag.
- A pre-release may be useful and downloadable while still being incomplete and bug-prone.
- Publish separate release assets per operating system so users download only what they need.
- Do not present unstable pre-releases as stable or store-ready.
- Do not present a macOS pre-release as notarized unless the workflow ran in `full` mode.
