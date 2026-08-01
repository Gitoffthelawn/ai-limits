# Testing

This document is the entry point for testing guidance.

Detailed checks stay in the documents that own the relevant behavior. Keep this page focused on cross-cutting rules, links, and platform-level test setup.

## Test Areas

- Provider source contract tests: [get-limits/providers/contract.md](../get-limits/providers/contract.md#tests).
- Notification trigger and delivery tests: [notifications](../notifications/testing.md#testing).
- Local desktop dev run: [dev run](../setup/dev-run.md).
- Local macOS debug build: [local build](../setup/local-build.md).
- Release artifact verification: `scripts/verify-macos-app.sh`.

The macOS permission reset procedure used before a clean permission check is documented in [macos-permission-reset.md](macos-permission-reset.md).

## Placement Rule

Put testing guidance where the tested behavior is defined when it is specific to one domain, provider, source, UI flow, or release artifact.

Use this document for:

- cross-cutting setup that affects several test areas;
- links to canonical detailed checks;
- shared manual-test conventions;
- OS-level reset or permission preparation.
