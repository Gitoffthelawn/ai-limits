# macOS GitHub Signing

## GitHub Actions Behavior

The desktop workflow builds macOS as a signed universal Apple app:

```text
npm exec tauri -- build --bundles app --target universal-apple-darwin
```

Workflow:

[Desktop build workflow](../../../.github/workflows/desktop-build.yml)

OS permission requirements:

[OS permissions](../../desktop/permissions.md)

Signing mode:

- unsigned macOS is not the current GitHub workflow path;
- signed macOS is the current GitHub workflow path;
- signing and notarization details must be checked against the workflow file before changing release expectations.

Default mode:

```text
full
```

Modes:

- `sign-only`: Developer ID signed, not notarized;
- `submit-only`: signed and submitted to Apple notarization without waiting for stapling;
- `full`: signed, notarized, and stapled.

First notarization for a new Apple Developer team can stay `In Progress` for hours or longer. After the first `Accepted` result, later `full` runs are usually much faster. See [GitHub builds](github-builds.md).

Required secrets are documented in [macos-signing-secrets.md](macos-signing-secrets.md).
