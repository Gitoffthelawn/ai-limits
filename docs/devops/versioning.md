# Versioning

Use SemVer Git tags for desktop releases:

```text
vMAJOR.MINOR.PATCH
```

The release channel carries release stability. Do not encode `stable`, `unstable`, or platform names in the tag.

The release process enforces:

- release input matches `vMAJOR.MINOR.PATCH`;
- release asset names include the version and platform.
- `CHANGELOG.md` contains a non-empty `Unreleased` section.

The release process turns the current `Unreleased` entries into the selected version, records the release date and link, and creates an annotated tag. The current implementation is [GitHub Actions](../../.github/workflows/desktop-build.yml).

Old `desktop-unstable-*` tags are historical and should not be used for new releases.
