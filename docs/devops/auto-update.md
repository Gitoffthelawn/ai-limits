# Auto-Update

Installed macOS clients update themselves from the published GitHub pre-release
without the user downloading a new disk image. Windows and Linux ship unsigned
and are not part of the update channel.

## Behaviour

- The frontend runs one update check at startup and then every 12 hours while
  the app stays open. Nothing is checked while the app is closed.
- A check downloads and installs an available update in the background. The
  running app keeps its current version until it is restarted.
- Once an update is staged, a banner offers `Restart now`. The app never
  restarts on its own.
- The `Automatic updates` toggle in Settings → Other controls whether checks run
  at all. It defaults to on. With it off, the app makes no update requests.
- A failed check (unreachable manifest, failed download, rejected signature) is
  silent; the next scheduled check retries.

The check itself is a plain request from the Rust side to the manifest URL, so
the webview Content Security Policy does not apply to it.

## Implementation

| Concern | Location |
| --- | --- |
| Plugin registration | [src-tauri/src/main.rs](../../src-tauri/src/main.rs) |
| Download and restart commands | [src-tauri/src/commands/app_update.rs](../../src-tauri/src/commands/app_update.rs) |
| Schedule, banner, failure handling | [frontend/modules/app-update.js](../../frontend/modules/app-update.js) |
| Check interval and default setting | [frontend/modules/constants.js](../../frontend/modules/constants.js) |
| Endpoint and public key | [src-tauri/tauri.conf.json](../../src-tauri/tauri.conf.json) |

## Signing

Update archives are signed with a minisign key pair that is separate from Apple
code signing and notarization. Apple signing proves the app is from a known
developer to macOS; this key proves to an already-installed client that an
update came from the same source as the copy it is replacing.

- The public key lives in `plugins.updater.pubkey` in
  [tauri.conf.json](../../src-tauri/tauri.conf.json) and is compiled into every
  build. It is not a secret.
- The private key is held only in the release workflow, as the repository
  secrets `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`,
  plus an offline backup outside this repository.
- Generate a pair with `npm exec tauri -- signer generate -w <path>`.

Losing the private key does not expose anything, but it does break the update
path: installed clients only accept signatures made by the key matching the
public key they already carry. Recovery requires publishing a build with a new
public key and having users install it manually once. Keep a backup.

## Manifest

The manifest is `updater/latest.json`, served from
the `main` branch over `raw.githubusercontent.com`. It always describes the
current release; it is overwritten rather than versioned. The GitHub
`releases/latest` URL is not usable here because every release is published as a
pre-release.

The release workflow generates it from the release tag, the signature produced
by the macOS build, and the `Unreleased` changelog section, then commits it in
the same commit as the finalized changelog. It is never edited by hand.

All three macOS platform keys point at the same universal archive. Windows and
Linux keys are absent, so clients on those platforms find no update. Because
`createUpdaterArtifacts` applies to every platform, those two builds turn it off
through [updater-disabled.conf.json](../../src-tauri/updater-disabled.conf.json);
otherwise they would demand the signing key to produce artifacts nothing reads.

The release tag and the `version` in `tauri.conf.json` must match; the workflow
refuses to release when they diverge, because clients compare their built-in
version against the manifest version.

## Test builds

Test builds are never offered to users, because the updater only reads the
manifest and the manifest only ever describes a promoted release. A build that
is not published through the release workflow does not appear there.

To exercise the full check → download → verify → install → restart path locally,
without a GitHub release, use
[scripts/serve-test-update.sh](../../scripts/serve-test-update.sh); run it with
`--help` for the procedure.

Before relying on the workflow-generated manifest for the first time, run one
real release and confirm an installed older build picks it up.
