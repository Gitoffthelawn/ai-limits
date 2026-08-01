# Cursor Usage Retrieval Options

## Known usage retrieval options

| Option | Plan/availability | Status | Notes |
|---|---|---|---|
| IDE backend `api2.cursor.sh` | Pro/Ultra/Team | Implemented | Stable internal endpoint used by Cursor; uses an access token after `cursor agent login`; its contract is not publicly documented |
| Cursor CLI `about/status` | Pro/Ultra/Team | Not implemented | Provides identity/auth/model/tier, but not billing usage; no CLI-based source exists in code |
| Dashboard API `cursor.com/api/...` | Any | Research-only | Requires web session cookie; high security risk |
| Admin API `api.cursor.com` | Enterprise | Official | Suitable for Enterprise monitoring; 403 expected on Pro/Teams without Enterprise |

---

## Recommendation

For personal Pro/Ultra/Team, the primary option is a locally authorized Cursor Agent and `api2.cursor.sh`. The endpoint is a stable internal Cursor endpoint, not a publicly documented provider contract, and requires a separate security review before production use.

For production/enterprise monitoring, the official Admin API is preferred when available for the plan and provides the required level of detail.

---

## Limitations

- `api2.cursor.sh` is a stable internal Cursor endpoint but has no publicly documented contract; `cursor.com/api/*` is also not publicly documented and either may change without notice
- the access token is short-lived
- the refresh token is a sensitive secret
- automated work with dashboard cookies should be disabled by default
