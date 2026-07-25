# Desktop DevOps Release Plan

## Global Plan

### 1. Confirm Local macOS Build

Status: done.

Outcome:

- Local macOS `.app` build is confirmed.
- Default DMG packaging is not confirmed.
- First GitHub Actions stage should use `.app`, not DMG, as the required macOS artifact.

### 2. Build GitHub Artifacts

Status: done.

See [GitHub builds](github-builds.md).

### 3. Generate Desktop Icons

Status: active.

See [Icons](icons.md).

### 4. Verify Downloaded Artifacts

Status: in progress.

See [Temporary artifact verification](artifact-verification-temp.md).

### 5. Publish Unstable GitHub Pre-Releases

Status: active.

See [GitHub releases](github-releases.md).

### 6. Add Installers and Broader Signed Distribution

Status: active for macOS, future for Windows and Linux.

See [GitHub releases](github-releases.md).

## Recommended Next Task

Verify downloaded unstable pre-release files on target platforms and document the results.
