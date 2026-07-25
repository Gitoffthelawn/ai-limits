# macOS Signing Secrets

Required for signing:

```text
APPLE_CERTIFICATE
APPLE_CERTIFICATE_PASSWORD
KEYCHAIN_PASSWORD
APPLE_TEAM_ID
```

Required for `submit-only` and `full` notarization:

```text
APPLE_API_KEY_CONTENT
APPLE_API_KEY_ID
APPLE_API_ISSUER
```

Example file:

[macOS signing secrets example](../../../scripts/macos-signing-secrets.example)

Do not set `APPLE_SIGNING_IDENTITY` in GitHub secrets when using `APPLE_CERTIFICATE`. The workflow imports the `.p12` certificate into a temporary keychain, and Tauri derives the signing identity from that certificate.
