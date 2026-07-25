# Cursor

## Current status

The PoC retrieves numeric Cursor usage/limits through the internal endpoint `api2.cursor.sh` and an access token created by `cursor agent login`. This is currently the only implemented Cursor source; there is no Cursor CLI fallback source in code.

If the token is not found, the request is rejected, or the response format has changed, the source reports the failure as unavailable data.

Research on `api2.cursor.sh`: [../../../product/analogs/cursor-api2-cursor-sh.md](../../../product/analogs/cursor-api2-cursor-sh.md).

---

## Provider Method: `cursor_api2_usage`

The primary PoC method retrieves numeric usage/limits through `api2.cursor.sh`.

The method:

- uses an access token after `cursor agent login`
- calls `GetCurrentPeriodUsage`
- returns included usage, usage percentages, and billing cycle
- depends on an unofficial Cursor backend contract
- requires a separate security review before production use

Endpoint details: [../../../product/analogs/cursor-api2-cursor-sh.md](../../../product/analogs/cursor-api2-cursor-sh.md).

Other known retrieval options are documented in [cursor-options.md](cursor-options.md).
