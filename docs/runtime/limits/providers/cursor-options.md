# Cursor Usage Retrieval Options

## Known usage retrieval options

| Option | Plan/availability | Status | Notes |
|---|---|---|---|
| IDE backend `api2.cursor.sh` | Pro/Ultra/Team | Implemented in PoC | Uses access token after `cursor agent login`; unofficial contract |
| Cursor CLI `about/status` | Pro/Ultra/Team | Not implemented | Provides identity/auth/model/tier, but not billing usage; no CLI-based source exists in code |
| Dashboard API `cursor.com/api/...` | Any | Research-only | Requires web session cookie; high security risk |
| Admin API `api.cursor.com` | Enterprise | Official | Suitable for Enterprise monitoring; 403 expected on Pro/Teams without Enterprise |

---

## Recommendation

For personal Pro/Ultra/Team, the primary PoC option is a locally authorized Cursor Agent and `api2.cursor.sh`. The method remains an unofficial provider method and requires a separate security review before production use.

For production/enterprise monitoring, the official Admin API is preferred when available for the plan and provides the required level of detail.

---

## Limitations

- `api2.cursor.sh` and `cursor.com/api/*` are not publicly documented contracts and may change without notice
- the access token is short-lived
- the refresh token is a sensitive secret
- automated work with dashboard cookies should be disabled by default
