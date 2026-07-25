# Artifact Verification

## Verify Downloaded Artifacts

Manually verify that downloaded release artifacts can be opened or installed on each target platform before pointing users at a pre-release:

```text
macOS:
  extract with ditto -x -k
  run scripts/verify-macos-app.sh
  launch the .app
  check signing and notarization mode / Gatekeeper UX

Windows:
  install or run the NSIS setup
  optionally install the MSI
  launch the app after installation

Linux:
  run the AppImage
  optionally install the DEB
  launch the app after installation
```

Do not change signing or notarization during artifact verification. Do not create GitHub Releases during artifact verification.

## macOS release artifact checks

Use `ditto` to extract the zip. Do not use `unzip`; it does not preserve macOS extended attributes and can break signed/stapled `.app` bundles.

```sh
mkdir -p /tmp/ai-limits-verify
ditto -x -k "AI Limits.app.zip" /tmp/ai-limits-verify
```

Run the shared verification script on the downloaded zip or extracted `.app`:

```sh
scripts/verify-macos-app.sh --notarization full "AI Limits.app.zip"
```

Or on the extracted bundle:

```sh
scripts/verify-macos-app.sh --notarization full "/tmp/ai-limits-verify/AI Limits.app"
```

The script runs:

```text
codesign -dv
codesign -d --entitlements -
codesign --verify --deep --strict
spctl --assess
xcrun stapler validate   # only in full mode
```

Verification should note the platform, the artifact file used, the installation/opening result, the launch result, and any blocking UX or security warning. Current results are recorded in [artifact-verification-results.md](artifact-verification-results.md).
