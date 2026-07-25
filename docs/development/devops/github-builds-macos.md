# GitHub Builds macOS Job

```text
runner: macos-latest
command: npm exec tauri -- build --bundles app --target universal-apple-darwin
artifact name: ai-limits-macos-app
artifact path: target/release/bundle/macos/AI Limits.app.zip
```

The workflow imports a Developer ID Application `.p12`, writes the App Store Connect API key, and lets Tauri sign the universal `.app` bundle. In `full` mode, Tauri also notarizes and staples before the zip is uploaded.

Entitlements: `src-tauri/Entitlements.plist` with hardened runtime enabled in `tauri.conf.json`.

The workflow verifies the final `.app` before archive upload:

```text
scripts/verify-macos-app.sh --notarization <mode> "target/universal-apple-darwin/release/bundle/macos/AI Limits.app"
```

In `full` mode, the script also verifies notarization and stapling:

```text
xcrun stapler validate "target/universal-apple-darwin/release/bundle/macos/AI Limits.app"
```

The `.app` bundle is archived with `ditto` after signing to preserve bundle structure, symlinks, and extended attributes:

```text
ditto -c -k --keepParent "AI Limits.app" "AI Limits.app.zip"
```

Do not use `--sequesterRsrc` for release archives. It moves extended attributes into `__MACOSX` AppleDouble files and can break stapled notarization tickets after extraction.

After archiving, the workflow extracts the zip with `ditto` and reruns the same verification on the round-tripped artifact:

```text
scripts/verify-macos-app.sh --notarization <mode> "target/release/bundle/macos/AI Limits.app.zip"
```

Local verification after download must also use `ditto`, not `unzip`. See [artifact verification](artifact-verification-temp.md).
