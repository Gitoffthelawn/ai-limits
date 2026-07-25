# Desktop DevOps

This directory is the source of truth for desktop build, artifacts, and release work until the desktop release pipeline is stable.

Detailed Tauri integration rules remain in [architecture](../../architecture.md) and [desktop docs](../../desktop/).

The step-by-step release plan is documented in [release-plan.md](release-plan.md).

## Goal

Build and publish desktop artifacts for the Tauri app across supported platforms:

- macOS — signed, notarized, stapled in CI (`full` mode by default)
- Windows — unsigned installer bundles
- Linux — unsigned `.deb` and `.AppImage`

The GitHub Actions workflow publishes those files through GitHub Releases for easier downloading. macOS DMG and Windows signing remain future work.

## Current Status

Confirmed:

- The app is a Rust + Tauri desktop application.
- `src/` is the shared Rust core and CLI.
- `src-tauri/` is the Tauri desktop adapter.
- Tauri uses structured data from the Rust core through commands.
- Tauri must not duplicate provider logic, limit semantics, or notification rules.
- Local macOS production `.app` build is confirmed.
- The produced macOS `.app` was manually launched and checked by the user.
- GitHub Actions desktop build workflow is implemented and verified.
- GitHub Actions produces a signed macOS `.app` artifact.
- GitHub Actions produces unsigned artifacts for Windows and Linux.
- GitHub build modes are documented as unsigned and signed.
- GitHub Actions artifacts were downloaded and inspected by file name/path.
- GitHub Actions publishes unstable GitHub pre-releases with desktop files for macOS, Windows, and Linux.

Known local macOS production build result:

```text
npm exec tauri -- build --bundles app
```

This produces:

```text
target/release/ai-limits-desktop
target/release/bundle/macos/AI Limits.app
```

Known issue:

- Default `npm exec tauri -- build` currently reaches DMG packaging and fails on the DMG bundling step.
- DMG is not a blocker for the current `.app` artifact stage.
- DMG packaging should be handled as a later, separate task.

Verified GitHub Actions run:

```text
URL: https://github.com/md2it/ai-limits/actions/runs/28758826398
Run ID: 28758826398
Trigger: workflow_dispatch
Status: success
```

## Scope

The desktop build/release work covers:

- GitHub Actions workflow design.
- Native OS builds on GitHub-hosted runners.
- macOS Developer ID signing and notarization in CI.
- Uploading build artifacts from GitHub Actions.
- Publishing unstable GitHub pre-releases from GitHub Actions.
- Documentation of build commands, produced paths, and known platform-specific issues.

Out of scope for now:

- macOS DMG as a required artifact.
- Windows code signing.
- Store distribution.
- Reworking Rust core logic.
- Reworking Tauri commands.
- Reworking frontend UI.
- Duplicating provider, config, limit, or notification logic in `src-tauri/`.
