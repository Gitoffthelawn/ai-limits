# Cursor

## Current status

The app retrieves numeric Cursor usage/limits through the stable internal endpoint `api2.cursor.sh`, which Cursor itself uses, and an access token created by `cursor agent login`. The endpoint has no publicly documented contract. This is currently the only implemented Cursor source; there is no Cursor CLI fallback source in code.

If the token is not found, the request is rejected, or the response format has changed, the source reports the failure as unavailable data.

---

## Provider Method: `cursor_api2_usage`

The primary method retrieves numeric usage/limits through `api2.cursor.sh`.

The method:

- uses an access token after `cursor agent login`
- calls `GetCurrentPeriodUsage`
- returns included usage, usage percentages, and billing cycle
- uses Cursor's stable internal endpoint, whose contract is not publicly documented
- requires a separate security review before production use

Code lives in `src/providers/cursor_api2/`:

- `mod.rs` — thin public facade (`collect_usage`) and re-export of `build_source_data`
- `fetch.rs` — Keychain token and HTTP request via `infra/os_access`
- `parse.rs` — scrape helpers and the internal `CursorApiFields` model
- `helpers.rs` — private date, amount, and billing helpers for projection
- `project.rs` — projection into `SourceData` (limits, billing, money) and package tests

Other known retrieval options are documented in [cursor-options.md](cursor-options.md).
