# Temporary Artifact Verification

## Verify Downloaded Artifacts

Status: in progress.

Plan:

- Verify that downloaded artifacts can be opened or installed on target platforms.
- Keep this as a manual verification step before GitHub Releases.
- Do not change signing or notarization during artifact verification.
- Do not create GitHub Releases during artifact verification.

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

Manual check checklist and current verification results are documented in [artifact-verification-results.md](artifact-verification-results.md).
