# macOS Signing

## Goal

A macOS artifact intended for users must be trusted by macOS: Developer ID signed, Apple-notarized, and stapled. The final archived artifact must be verified after packaging, because packaging can invalidate the properties being protected.

This applies to every layer the user receives. macOS assesses a downloaded disk image on its own, so the disk image needs its own notarization ticket and is not covered by the ticket stapled to the application bundle inside it. The disk image is built, signed, notarized, and stapled as a separate step from the application bundle, and both layers are verified.

The disk image is not built by Tauri's own dmg bundler. That bundler styles the window (background, icon positions) through a Finder AppleScript step, which requires the Automation permission to control Finder. That permission is unavailable on GitHub Actions runners and fails the step silently rather than failing the build, so the shipped disk image would be unstyled with no indication anything went wrong — this happened undetected across the first two releases with a disk image. [scripts/build-macos-dmg.sh](../../scripts/build-macos-dmg.sh) writes the same window layout directly instead, using no Finder automation.

## Modes

- `full` is the release-ready mode: signing, notarization, and stapling are complete.
- `submit-only` is for an early notarization submission; it is not a user-ready artifact.
- `sign-only` is for build or signing diagnostics; macOS may warn users about the artifact.

Notarization time is controlled by Apple and can be materially longer for a new team. A release must not be represented as notarized until Apple accepts it.

## Credential Policy

The protected build environment receives the signing certificate and Apple notarization credentials. The repository contains neither credentials nor their values. The platform implementation must derive the signing identity from the supplied certificate rather than from a hard-coded identity.

The current implementation is the [desktop workflow](../../.github/workflows/desktop-build.yml), [Tauri packaging configuration](../../src-tauri/tauri.conf.json), [disk image build script](../../scripts/build-macos-dmg.sh), and [macOS verification script](../../scripts/verify-macos-app.sh). The [secrets example](../../scripts/macos-signing-secrets.example) lists the current integration variables.
