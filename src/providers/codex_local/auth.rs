use std::fs;
use std::io;
use std::path::Path;

use chrono::{DateTime, Duration, Utc};
use serde_json::Value;

const AUTH_FILE: &str = "auth.json";
const CLAIMS_KEY: &str = "https://api.openai.com/auth";
const START_CLAIM: &str = "chatgpt_subscription_active_start";
const UNTIL_CLAIM: &str = "chatgpt_subscription_active_until";
const PLAN_CLAIM: &str = "chatgpt_plan_type";
const LAST_CHECKED_CLAIM: &str = "chatgpt_subscription_last_checked";
const PLAN_NAME_MAX_CHARS: usize = 32;
const UNUSABLE_PLAN_NOTE: &str = "plan name: the auth token claim is not a usable plan name";
const RENEWAL_EXPIRY_GRACE_MINUTES: i64 = 2;
const RENEWAL_NOTE: &str =
    "renewal date is the subscription active-until date; for a cancelled subscription it is the end of access, not a renewal";
const EXPIRED_RENEWAL_NOTE: &str =
    "renewal date rejected because the local auth token's subscription window has already ended and could not be confirmed as current";

/// Subscription facts read from the local Codex auth token.
///
/// Only the subscription timestamps and the plan name ever leave this module;
/// no token, identifier, other claim value, or file content is carried into
/// `diagnostics`.
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct CodexLocalSubscription {
    pub(super) started_at: Option<String>,
    pub(super) renewal_at: Option<String>,
    /// Offline fallback for `account.plan`; a limits snapshot wins over it.
    pub(super) plan: Option<String>,
    pub(super) diagnostics: Vec<String>,
}

pub(super) fn read_subscription(root: &Path, now: DateTime<Utc>) -> CodexLocalSubscription {
    let content = match fs::read_to_string(root.join(AUTH_FILE)) {
        Ok(content) => content,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return failed("subscription dates: auth.json not found");
        }
        Err(_) => return failed("subscription dates: auth.json is unreadable"),
    };

    let Some(claims) = auth_claims(&content) else {
        return failed("subscription dates: auth token payload could not be decoded");
    };

    let mut diagnostics = Vec::new();
    let started_at = claim_datetime(
        &claims,
        START_CLAIM,
        "subscription start date",
        &mut diagnostics,
    );
    let renewal_at = claim_datetime(&claims, UNTIL_CLAIM, "renewal date", &mut diagnostics)
        .and_then(|renewal| confirm_future_renewal(renewal, now, &mut diagnostics));
    let plan = plan_name(&claims, &mut diagnostics);
    note_subscription_age(&claims, now, &mut diagnostics);

    CodexLocalSubscription {
        started_at: started_at.map(format_utc),
        renewal_at: renewal_at.map(format_utc),
        plan,
        diagnostics,
    }
}

/// The plan name is the only claim value that leaves this module, so it is
/// accepted only in the short token-like shape a plan name actually has.
fn plan_name(claims: &Value, diagnostics: &mut Vec<String>) -> Option<String> {
    let value = claims.get(PLAN_CLAIM).and_then(Value::as_str)?;
    let usable = !value.is_empty()
        && value.chars().count() <= PLAN_NAME_MAX_CHARS
        && value
            .chars()
            .all(|item| item.is_ascii_alphanumeric() || item == '-' || item == '_');

    if !usable {
        diagnostics.push(UNUSABLE_PLAN_NOTE.to_string());
        return None;
    }

    Some(value.to_string())
}

/// `chatgpt_subscription_last_checked` marks how fresh the subscription claims
/// are, not when this run collected them, so it is reported as an age and
/// never becomes `collected_at` or `data_as_of`.
fn note_subscription_age(claims: &Value, now: DateTime<Utc>, diagnostics: &mut Vec<String>) {
    let Some(value) = claims.get(LAST_CHECKED_CLAIM).and_then(Value::as_str) else {
        return;
    };

    let Ok(last_checked) = DateTime::parse_from_rfc3339(value) else {
        diagnostics.push("subscription data age: timestamp could not be parsed".to_string());
        return;
    };

    let age = now - last_checked.with_timezone(&Utc);
    diagnostics.push(format!(
        "subscription data from the local auth token was last checked {}",
        format_age(age)
    ));
}

fn format_age(age: Duration) -> String {
    if age < Duration::zero() {
        return "in the future".to_string();
    }

    match (age.num_days(), age.num_hours(), age.num_minutes()) {
        (days, _, _) if days > 0 => format!("{days}d ago"),
        (_, hours, _) if hours > 0 => format!("{hours}h ago"),
        (_, _, minutes) if minutes > 0 => format!("{minutes}m ago"),
        _ => "less than a minute ago".to_string(),
    }
}

/// The claims describe the subscription window as of token issuance and go
/// stale between token refreshes. A window that has already ended cannot be
/// told apart from a cancelled subscription using local data alone, so it is
/// never presented as an upcoming renewal.
fn confirm_future_renewal(
    renewal: DateTime<Utc>,
    now: DateTime<Utc>,
    diagnostics: &mut Vec<String>,
) -> Option<DateTime<Utc>> {
    if renewal < now - Duration::minutes(RENEWAL_EXPIRY_GRACE_MINUTES) {
        diagnostics.push(EXPIRED_RENEWAL_NOTE.to_string());
        return None;
    }

    diagnostics.push(RENEWAL_NOTE.to_string());
    Some(renewal)
}

fn failed(diagnostic: &str) -> CodexLocalSubscription {
    CodexLocalSubscription {
        diagnostics: vec![diagnostic.to_string()],
        ..CodexLocalSubscription::default()
    }
}

fn auth_claims(content: &str) -> Option<Value> {
    let id_token = serde_json::from_str::<Value>(content)
        .ok()?
        .pointer("/tokens/id_token")?
        .as_str()?
        .to_string();

    let mut segments = id_token.split('.');
    let (_, payload, _) = (segments.next()?, segments.next()?, segments.next()?);
    if segments.next().is_some() {
        return None;
    }

    let decoded = decode_base64url(payload)?;
    let payload = serde_json::from_slice::<Value>(&decoded).ok()?;
    payload.get(CLAIMS_KEY).cloned()
}

fn claim_datetime(
    claims: &Value,
    claim: &str,
    label: &str,
    diagnostics: &mut Vec<String>,
) -> Option<DateTime<Utc>> {
    let Some(value) = claims.get(claim).and_then(Value::as_str) else {
        diagnostics.push(format!("{label}: claim is missing from the auth token"));
        return None;
    };

    match DateTime::parse_from_rfc3339(value) {
        Ok(parsed) => Some(parsed.with_timezone(&Utc)),
        Err(_) => {
            diagnostics.push(format!("{label}: timestamp could not be parsed"));
            None
        }
    }
}

fn format_utc(value: DateTime<Utc>) -> String {
    value.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn decode_base64url(input: &str) -> Option<Vec<u8>> {
    let mut decoded = Vec::with_capacity(input.len() / 4 * 3);
    let mut buffer = 0u32;
    let mut bits = 0u32;

    for byte in input.bytes() {
        if byte == b'=' {
            break;
        }

        let value = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'-' => 62,
            b'_' => 63,
            _ => return None,
        };

        buffer = (buffer << 6) | u32::from(value);
        bits += 6;

        if bits >= 8 {
            bits -= 8;
            decoded.push((buffer >> bits) as u8);
        }
    }

    Some(decoded)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::*;

    const SECRET: &str = "sUpErSeCrEtTokenBytes";

    fn now() -> DateTime<Utc> {
        "2026-07-01T00:00:00Z".parse().expect("valid timestamp")
    }

    fn temp_root(suffix: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "ai-limits-codex-auth-{}-{suffix}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create temp root");
        root
    }

    fn write_auth(root: &Path, id_token: &str) {
        let content = serde_json::json!({
            "OPENAI_API_KEY": Value::Null,
            "tokens": {
                "access_token": SECRET,
                "id_token": id_token,
                "refresh_token": SECRET,
            }
        });
        fs::write(root.join(AUTH_FILE), content.to_string()).expect("write auth.json");
    }

    fn id_token(claims: Value) -> String {
        format!("header.{}.signature", encode_base64url(&claims.to_string()))
    }

    fn encode_base64url(input: &str) -> String {
        const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
        let bytes = input.as_bytes();
        let mut encoded = String::new();

        for chunk in bytes.chunks(3) {
            let mut buffer = 0u32;
            for (index, byte) in chunk.iter().enumerate() {
                buffer |= u32::from(*byte) << (16 - 8 * index);
            }
            for index in 0..chunk.len() + 1 {
                encoded.push(ALPHABET[((buffer >> (18 - 6 * index)) & 0x3F) as usize] as char);
            }
        }

        encoded
    }

    #[test]
    fn reads_subscription_dates_from_namespaced_claims() {
        let root = temp_root("dates");
        write_auth(
            &root,
            &id_token(serde_json::json!({
                CLAIMS_KEY: {
                    START_CLAIM: "2026-06-02T11:47:15+00:00",
                    UNTIL_CLAIM: "2026-08-02T11:47:15+00:00",
                    "chatgpt_plan_type": "plus"
                }
            })),
        );

        let subscription = read_subscription(&root, now());

        assert_eq!(
            subscription.started_at.as_deref(),
            Some("2026-06-02T11:47:15Z")
        );
        assert_eq!(
            subscription.renewal_at.as_deref(),
            Some("2026-08-02T11:47:15Z")
        );
        assert_eq!(subscription.diagnostics, vec![RENEWAL_NOTE.to_string()]);

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn reads_the_plan_claim_and_reports_the_subscription_age() {
        let root = temp_root("plan");
        write_auth(
            &root,
            &id_token(serde_json::json!({
                CLAIMS_KEY: {
                    PLAN_CLAIM: "plus",
                    LAST_CHECKED_CLAIM: "2026-06-27T00:00:00+00:00"
                }
            })),
        );

        let subscription = read_subscription(&root, now());

        assert_eq!(subscription.plan.as_deref(), Some("plus"));
        assert!(subscription.diagnostics.contains(
            &"subscription data from the local auth token was last checked 4d ago".to_string()
        ));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn unusable_plan_claim_degrades_to_null() {
        let root = temp_root("bad-plan");
        write_auth(
            &root,
            &id_token(serde_json::json!({
                CLAIMS_KEY: { PLAN_CLAIM: format!("plus {SECRET}") }
            })),
        );

        let subscription = read_subscription(&root, now());

        assert_eq!(subscription.plan, None);
        assert!(subscription
            .diagnostics
            .contains(&UNUSABLE_PLAN_NOTE.to_string()));
        for entry in &subscription.diagnostics {
            assert!(!entry.contains(SECRET));
        }

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn unparseable_last_checked_claim_degrades_to_a_generic_note() {
        let root = temp_root("bad-last-checked");
        write_auth(
            &root,
            &id_token(serde_json::json!({
                CLAIMS_KEY: { LAST_CHECKED_CLAIM: format!("{SECRET}-not-a-date") }
            })),
        );

        let subscription = read_subscription(&root, now());

        assert!(subscription
            .diagnostics
            .contains(&"subscription data age: timestamp could not be parsed".to_string()));
        for entry in &subscription.diagnostics {
            assert!(!entry.contains(SECRET));
        }

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn missing_auth_file_degrades_to_null() {
        let root = temp_root("missing");

        let subscription = read_subscription(&root, now());

        assert_eq!(subscription.started_at, None);
        assert_eq!(subscription.renewal_at, None);
        assert_eq!(
            subscription.diagnostics,
            vec!["subscription dates: auth.json not found".to_string()]
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn missing_claims_degrade_to_null() {
        let root = temp_root("no-claims");
        write_auth(
            &root,
            &id_token(serde_json::json!({ CLAIMS_KEY: { "chatgpt_plan_type": "plus" } })),
        );

        let subscription = read_subscription(&root, now());

        assert_eq!(subscription.started_at, None);
        assert_eq!(subscription.renewal_at, None);
        assert!(subscription
            .diagnostics
            .contains(&"renewal date: claim is missing from the auth token".to_string()));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn past_renewal_is_rejected_and_start_date_is_kept() {
        let root = temp_root("stale-renewal");
        write_auth(
            &root,
            &id_token(serde_json::json!({
                CLAIMS_KEY: {
                    START_CLAIM: "2026-06-02T11:47:15+00:00",
                    UNTIL_CLAIM: "2026-08-02T11:47:15+00:00"
                }
            })),
        );

        let subscription = read_subscription(
            &root,
            "2026-08-03T03:24:01Z".parse().expect("valid timestamp"),
        );

        assert_eq!(subscription.renewal_at, None);
        assert_eq!(
            subscription.started_at.as_deref(),
            Some("2026-06-02T11:47:15Z")
        );
        assert_eq!(
            subscription.diagnostics,
            vec![EXPIRED_RENEWAL_NOTE.to_string()]
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn future_renewal_survives_with_the_ambiguity_note() {
        let root = temp_root("future-renewal");
        write_auth(
            &root,
            &id_token(serde_json::json!({
                CLAIMS_KEY: {
                    START_CLAIM: "2026-06-02T11:47:15+00:00",
                    UNTIL_CLAIM: "2026-08-02T11:47:15+00:00"
                }
            })),
        );

        let subscription = read_subscription(
            &root,
            "2026-08-02T11:00:00Z".parse().expect("valid timestamp"),
        );

        assert_eq!(
            subscription.renewal_at.as_deref(),
            Some("2026-08-02T11:47:15Z")
        );
        assert_eq!(subscription.diagnostics, vec![RENEWAL_NOTE.to_string()]);

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn renewal_within_clock_grace_remains_usable() {
        let root = temp_root("renewal-grace");
        write_auth(
            &root,
            &id_token(
                serde_json::json!({ CLAIMS_KEY: { UNTIL_CLAIM: "2026-08-02T11:47:15+00:00" } }),
            ),
        );

        let subscription = read_subscription(
            &root,
            "2026-08-02T11:49:14Z".parse().expect("valid timestamp"),
        );

        assert_eq!(
            subscription.renewal_at.as_deref(),
            Some("2026-08-02T11:47:15Z")
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn unparseable_timestamp_degrades_to_null() {
        let root = temp_root("bad-timestamp");
        write_auth(
            &root,
            &id_token(serde_json::json!({
                CLAIMS_KEY: {
                    START_CLAIM: "not-a-date",
                    UNTIL_CLAIM: "2026-08-02T11:47:15+00:00"
                }
            })),
        );

        let subscription = read_subscription(&root, now());

        assert_eq!(subscription.started_at, None);
        assert_eq!(
            subscription.renewal_at.as_deref(),
            Some("2026-08-02T11:47:15Z")
        );
        assert!(subscription
            .diagnostics
            .contains(&"subscription start date: timestamp could not be parsed".to_string()));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn malformed_token_never_appears_in_diagnostics() {
        let root = temp_root("malformed");
        write_auth(&root, &format!("header.{SECRET}!!!.signature"));

        let subscription = read_subscription(&root, now());

        assert_eq!(subscription.started_at, None);
        assert_eq!(subscription.renewal_at, None);
        assert_eq!(
            subscription.diagnostics,
            vec!["subscription dates: auth token payload could not be decoded".to_string()]
        );
        for entry in &subscription.diagnostics {
            assert!(!entry.contains(SECRET));
            assert!(!entry.contains("header"));
            assert!(!entry.contains("signature"));
        }

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn invalid_auth_json_degrades_to_null() {
        let root = temp_root("bad-json");
        fs::write(root.join(AUTH_FILE), format!("{{\"tokens\": {SECRET}"))
            .expect("write auth.json");

        let subscription = read_subscription(&root, now());

        assert_eq!(
            subscription.diagnostics,
            vec!["subscription dates: auth token payload could not be decoded".to_string()]
        );
        assert!(!subscription.diagnostics[0].contains(SECRET));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn decodes_base64url_payload_without_padding() {
        assert_eq!(decode_base64url("eyJhIjoxfQ"), Some(b"{\"a\":1}".to_vec()));
        assert_eq!(decode_base64url("_-_-"), Some(vec![0xff, 0xef, 0xfe]));
        assert_eq!(decode_base64url("bad!"), None);
    }
}
