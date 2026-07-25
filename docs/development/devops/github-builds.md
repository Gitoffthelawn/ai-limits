# GitHub Builds

Job details are documented in [github-builds-jobs.md](github-builds-jobs.md) and [github-builds-macos.md](github-builds-macos.md).

## Desktop build workflow

Status: active.

GitHub Actions workflow:

[Desktop build workflow](../../../.github/workflows/desktop-build.yml)

Workflow name:

```text
Desktop build
```

Trigger:

```text
workflow_dispatch
```

No automatic `push`, `pull_request`, or tag trigger is included.

When starting the workflow, choose **macOS notarization mode**:

| Mode | What happens | CI time | Release-ready macOS |
|------|----------------|---------|---------------------|
| `full` (default) | Sign, notarize, wait for Apple, staple | minutes to hours | yes |
| `submit-only` | Sign, submit notarization, do not wait | ~5 min | no (check Apple later) |
| `sign-only` | Developer ID sign only | ~3 min | no |

Use `full` for pre-releases that users download from GitHub. Use `submit-only` or `sign-only` only when iterating on CI and you do not need a finished macOS artifact yet.

When publishing a GitHub Release, provide the release version. The workflow validates it, finalizes the current `Unreleased` section in `CHANGELOG.md`, and publishes the matching tag and release. See [Versioning](versioning.md).

### macOS notarization notes

- First notarization for a new Apple Developer team can stay `In Progress` at Apple for hours or longer while the app is held for in-depth analysis.
- After the first `Accepted` result, later `full` runs are usually much faster.
- Check submission status locally:

```sh
xcrun notarytool history --key ... --key-id ... --issuer ...
```

- If you used `submit-only` and Apple later reports `Accepted`, either rerun the workflow with `full` or staple the same `.app` locally with `xcrun stapler staple`.

See [macOS signing](macos-signing.md) for secrets and signing details.

## Verification result

- Workflow starts manually from GitHub Actions.
- macOS, Windows, and Linux jobs pass when secrets and runners are available.
- Artifacts are created and uploaded for all three platforms.
- Release publishing creates an annotated SemVer tag and unstable GitHub pre-release after all platform jobs pass.
- macOS signing is used.
- macOS notarization is controlled by workflow input.
- Windows and Linux signing are not used.

## Implementation guardrails

- Do not change Rust core code in `src/`.
- Do not change Tauri command behavior in `src-tauri/src/`.
- Do not change frontend UI behavior.
- Do not change provider, limit, config, or notification logic.
- Keep Windows and Linux signing out of the current workflow unless explicitly requested.
- Keep macOS signing secrets in GitHub Actions only; do not commit them.
- Keep unstable release publishing clear about the selected macOS notarization mode.
