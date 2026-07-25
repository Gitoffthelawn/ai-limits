# GitHub Builds Jobs

Job order:

```text
validate-release-version -> rust-format -> build-macos, build-windows, build-linux -> publish-unstable-release
```

- `validate-release-version` checks the release runs from `main`, that the version input matches `vMAJOR.MINOR.PATCH`, that the tag and release do not already exist, and that `CHANGELOG.md` has a non-empty `Unreleased` section.
- `rust-format` runs `cargo fmt --all -- --check` and gates all platform build jobs.

Verified jobs:

```text
build-macos:   signed macOS app, notarization verified when full, artifact uploaded
build-windows: passed, artifact uploaded
build-linux:   passed, artifact uploaded
```

Common GitHub job setup:

- checkout repository;
- install Node.js 22;
- install Rust stable through `dtolnay/rust-toolchain@stable`;
- install npm dependencies with `npm ci`;
- upload artifacts with `actions/upload-artifact@v4`;
- keep artifact retention at 14 days.

### Windows job

```text
runner: windows-latest
command: npm exec tauri -- build --bundles nsis,msi
artifact name: ai-limits-windows-unsigned
artifact paths:
  target/release/bundle/nsis/*.exe
  target/release/bundle/msi/*.msi
```

Windows signing is not included.

### Linux job

```text
runner: ubuntu-latest
command: npm exec tauri -- build --bundles deb,appimage
artifact name: ai-limits-linux-unsigned
artifact paths:
  target/release/bundle/deb/*.deb
  target/release/bundle/appimage/*.AppImage
```

Linux system dependencies added to the workflow:

```text
libwebkit2gtk-4.1-dev
libgtk-3-dev
libayatana-appindicator3-dev
librsvg2-dev
patchelf
```
