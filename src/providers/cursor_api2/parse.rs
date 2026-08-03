use std::collections::HashSet;

use serde_json::{Map, Value};

use super::fetch::CursorResponses;
use super::helpers::{parse_price, PlanPrice};

pub(super) const PAGE_SIZE: u64 = 1000;

/// The whole field model the projection works from.
///
/// Every value here was read from a named path inside a named response. No
/// value is ever located by searching the whole body for a key name, because
/// `limit`, `price`, and the token counters all occur in several unrelated
/// objects of these responses.
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct CursorFields {
    pub plan: Option<String>,
    pub price: Option<PlanPrice>,
    pub renewal_at_ms: Option<i64>,

    pub total_percent_used: Option<f64>,
    pub auto_percent_used: Option<f64>,
    pub api_percent_used: Option<f64>,
    pub included_spend_cents: Option<f64>,
    pub plan_limit_cents: Option<f64>,
    pub billing_cycle_start_ms: Option<i64>,
    pub billing_cycle_end_ms: Option<i64>,
    pub individual_used: Option<f64>,

    pub usage_based_allowed: Option<bool>,
    pub hard_limit: Option<f64>,

    pub tokens: TokenFacts,
    pub activity: ActivityFacts,

    pub diagnostics: Vec<String>,
}

impl CursorFields {
    pub(super) fn is_empty(&self) -> bool {
        self.plan.is_none()
            && self.price.is_none()
            && self.renewal_at_ms.is_none()
            && self.total_percent_used.is_none()
            && self.auto_percent_used.is_none()
            && self.api_percent_used.is_none()
            && self.included_spend_cents.is_none()
            && self.plan_limit_cents.is_none()
            && self.billing_cycle_end_ms.is_none()
            && self.hard_limit.is_none()
            && self.tokens == TokenFacts::default()
            && self.activity == ActivityFacts::default()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct TokenFacts {
    pub input: Option<u64>,
    pub output: Option<u64>,
    pub cache_read: Option<u64>,
    pub cache_write: Option<u64>,
}

impl TokenFacts {
    /// The response carries no total token field, so the total is a sum. A sum
    /// over an unknown subset would understate the figure, so a missing
    /// component leaves the total unknown instead.
    pub(super) fn total(&self) -> Option<u64> {
        let input = self.input?;
        let output = self.output?;
        let cache_read = self.cache_read?;
        let cache_write = self.cache_write?;
        input
            .checked_add(output)?
            .checked_add(cache_read)?
            .checked_add(cache_write)
    }

    pub(super) fn is_partial(&self) -> bool {
        *self != Self::default() && self.total().is_none()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct ActivityFacts {
    pub events_count: Option<u64>,
    pub sessions_count: Option<u64>,
    pub latest_activity_ms: Option<i64>,
    pub sessions_incomplete: bool,
}

/// Accumulates `GetFilteredUsageEvents` pages.
///
/// `conversationId` never leaves this type: it is used to count distinct
/// values, and only that count is published.
#[derive(Clone, Debug, Default)]
pub struct UsageEventPages {
    total_count: Option<u64>,
    collected: u64,
    conversations: HashSet<String>,
    missing_conversation_id: bool,
    latest_ms: Option<i64>,
    bodies: Vec<String>,
    failed: bool,
    capped: bool,
    unreadable: bool,
}

impl UsageEventPages {
    /// Adds one page and reports whether paging should continue.
    pub(super) fn add_page(&mut self, body: &str) -> bool {
        let Ok(Value::Object(page)) = serde_json::from_str::<Value>(body) else {
            self.unreadable = true;
            return false;
        };

        if self.total_count.is_none() {
            self.total_count = unsigned_at(&page, &["totalUsageEventsCount"]);
        }

        let events = page
            .get("usageEventsDisplay")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        for event in &events {
            self.collected += 1;

            match event.get("conversationId") {
                Some(Value::String(value)) if !value.is_empty() => {
                    self.conversations.insert(value.clone());
                }
                Some(Value::Number(value)) => {
                    self.conversations.insert(value.to_string());
                }
                _ => self.missing_conversation_id = true,
            }

            if let Some(timestamp) = event
                .as_object()
                .and_then(|event| integer_at(event, &["timestamp"]))
            {
                self.latest_ms = Some(
                    self.latest_ms
                        .map_or(timestamp, |latest| latest.max(timestamp)),
                );
            }
        }

        self.bodies.push(body.to_string());

        !events.is_empty() && !self.is_complete()
    }

    pub(super) fn mark_failed(&mut self) {
        self.failed = true;
    }

    pub(super) fn mark_capped(&mut self) {
        self.capped = true;
    }

    pub(super) fn is_complete(&self) -> bool {
        !self.failed && !self.capped && !self.unreadable && self.total_count == Some(self.collected)
    }

    pub(super) fn was_requested(&self) -> bool {
        !self.bodies.is_empty() || self.failed || self.unreadable
    }

    pub(super) fn bodies(&self) -> &[String] {
        &self.bodies
    }

    pub(super) fn facts(&self) -> ActivityFacts {
        let complete = self.is_complete() && !self.missing_conversation_id;

        ActivityFacts {
            events_count: self.total_count,
            sessions_count: complete.then_some(self.conversations.len() as u64),
            latest_activity_ms: self.latest_ms,
            sessions_incomplete: self.was_requested() && !complete,
        }
    }
}

/// Assembles the field model from the collected responses.
pub(super) fn assemble(responses: &CursorResponses) -> CursorFields {
    let mut fields = CursorFields::default();

    for diagnostic in &responses.fetch_diagnostics {
        fields.diagnostics.push((*diagnostic).to_string());
    }

    match object_response(responses.plan_info.as_ref()) {
        Some(body) => read_plan_info(&body, &mut fields),
        None => fields
            .diagnostics
            .push("plan and price: the plan response could not be read".to_string()),
    }

    match object_response(responses.current_period.as_ref()) {
        Some(body) => read_current_period(&body, &mut fields),
        None => fields
            .diagnostics
            .push("plan limits: the current period response could not be read".to_string()),
    }

    match object_response(responses.hard_limit.as_ref()) {
        Some(body) => {
            fields.usage_based_allowed =
                boolean_at(&body, &["noUsageBasedAllowed"]).map(|value| !value);
            fields.hard_limit = number_at(&body, &["hardLimit"]);
        }
        None => fields
            .diagnostics
            .push("on-demand spend limit: the hard limit response could not be read".to_string()),
    }

    match object_response(responses.aggregated.as_ref()) {
        Some(body) => {
            fields.tokens = TokenFacts {
                input: unsigned_at(&body, &["totalInputTokens"]),
                output: unsigned_at(&body, &["totalOutputTokens"]),
                cache_read: unsigned_at(&body, &["totalCacheReadTokens"]),
                cache_write: unsigned_at(&body, &["totalCacheWriteTokens"]),
            };
        }
        None => fields
            .diagnostics
            .push("token usage: the aggregated usage response could not be read".to_string()),
    }

    if fields.tokens.is_partial() {
        fields.diagnostics.push(
            "total tokens: a component of the sum is missing, so no total is reported".to_string(),
        );
    }

    fields.activity = responses.filtered.facts();
    if fields.activity.sessions_incomplete {
        fields.diagnostics.push(
            "sessions: the usage event pages are incomplete, so no session count is reported"
                .to_string(),
        );
    }

    fields
}

fn read_plan_info(body: &Map<String, Value>, fields: &mut CursorFields) {
    let Some(plan_info) = body.get("planInfo").and_then(Value::as_object) else {
        fields
            .diagnostics
            .push("plan and price: the plan response has no plan information".to_string());
        return;
    };

    fields.plan = string_at(plan_info, &["planName"]);
    fields.renewal_at_ms = integer_at(plan_info, &["billingCycleEnd"]);

    match string_at(plan_info, &["price"]).as_deref().map(parse_price) {
        Some(Some(price)) => fields.price = Some(price),
        Some(None) => fields
            .diagnostics
            .push("price: the reported price string is not in a recognized form".to_string()),
        None => fields
            .diagnostics
            .push("price: the plan response reports no price".to_string()),
    }
}

fn read_current_period(body: &Map<String, Value>, fields: &mut CursorFields) {
    fields.billing_cycle_start_ms = integer_at(body, &["billingCycleStart"]);
    fields.billing_cycle_end_ms = integer_at(body, &["billingCycleEnd"]);
    fields.individual_used = number_at(body, &["spendLimitUsage", "individualUsed"]);

    if body.get("planUsage").and_then(Value::as_object).is_none() {
        return;
    }

    fields.total_percent_used = number_at(body, &["planUsage", "totalPercentUsed"]);
    fields.auto_percent_used = number_at(body, &["planUsage", "autoPercentUsed"]);
    fields.api_percent_used = number_at(body, &["planUsage", "apiPercentUsed"]);
    fields.included_spend_cents = number_at(body, &["planUsage", "includedSpend"]);
    fields.plan_limit_cents = number_at(body, &["planUsage", "limit"]);
}

pub(super) fn plan_usage_is_present(body: &str) -> bool {
    object(body)
        .and_then(|body| {
            body.get("planUsage")
                .and_then(Value::as_object)
                .map(|plan_usage| !plan_usage.is_empty())
        })
        .unwrap_or(false)
}

pub(super) fn billing_window(body: &str) -> Option<(i64, i64)> {
    let body = object(body)?;
    Some((
        integer_at(&body, &["billingCycleStart"])?,
        integer_at(&body, &["billingCycleEnd"])?,
    ))
}

pub(super) fn monthly_billing_window(body: &str) -> Option<(i64, i64)> {
    let body = object(body)?;
    Some((
        integer_at(&body, &["startDateEpochMillis"])?,
        integer_at(&body, &["endDateEpochMillis"])?,
    ))
}

pub(super) fn is_enterprise_account(body: &str) -> Option<bool> {
    boolean_at(&object(body)?, &["isEnterpriseUser"])
}

pub(super) fn team_id(body: &str) -> Option<i64> {
    integer_at(&object(body)?, &["teamId"]).filter(|team| *team != 0)
}

/// Builds the raw artifact from the collected responses.
///
/// `GetMe` is never included: its only useful fields are the team identity, and
/// the rest of it is account identity. Everything else passes through
/// [`sanitize`], which drops identifier-shaped keys entirely.
pub(super) fn sanitized_raw(responses: &CursorResponses) -> Option<String> {
    let mut raw = Map::new();

    for (method, outcome) in [
        ("GetPlanInfo", responses.plan_info.as_ref()),
        ("GetCurrentPeriodUsage", responses.current_period.as_ref()),
        ("GetHardLimit", responses.hard_limit.as_ref()),
        ("GetAggregatedUsageEvents", responses.aggregated.as_ref()),
    ] {
        if let Some(body) = outcome.and_then(|outcome| outcome.as_ref().ok()) {
            if let Ok(value) = serde_json::from_str::<Value>(body) {
                raw.insert(method.to_string(), sanitize(value));
            }
        }
    }

    let pages: Vec<Value> = responses
        .filtered
        .bodies()
        .iter()
        .filter_map(|body| serde_json::from_str::<Value>(body).ok())
        .map(sanitize)
        .collect();
    if !pages.is_empty() {
        raw.insert("GetFilteredUsageEvents".to_string(), Value::Array(pages));
    }

    if raw.is_empty() {
        return None;
    }

    serde_json::to_string_pretty(&Value::Object(raw)).ok()
}

/// Removes account identifiers from a response before it can be published.
fn sanitize(value: Value) -> Value {
    match value {
        Value::Object(entries) => Value::Object(
            entries
                .into_iter()
                .filter(|(key, _)| !is_identifier_key(key))
                .map(|(key, value)| (key, sanitize(value)))
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.into_iter().map(sanitize).collect()),
        Value::String(text) if looks_like_an_email(&text) => Value::Null,
        other => other,
    }
}

fn is_identifier_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();

    key.ends_with("id")
        || key.ends_with("ids")
        || key.contains("email")
        || key.contains("user")
        || key.contains("auth")
        || key.contains("secret")
        || key.contains("password")
        || (key.ends_with("token") && !key.ends_with("tokens"))
}

fn looks_like_an_email(text: &str) -> bool {
    let Some((local, domain)) = text.split_once('@') else {
        return false;
    };

    !local.is_empty() && domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.')
}

fn object_response(outcome: Option<&Result<String, &'static str>>) -> Option<Map<String, Value>> {
    object(outcome?.as_ref().ok()?)
}

fn object(body: &str) -> Option<Map<String, Value>> {
    match serde_json::from_str::<Value>(body) {
        Ok(Value::Object(entries)) => Some(entries),
        _ => None,
    }
}

fn at<'a>(body: &'a Map<String, Value>, path: &[&str]) -> Option<&'a Value> {
    let (first, rest) = path.split_first()?;
    let mut current = body.get(*first)?;

    for step in rest {
        current = current.get(*step)?;
    }

    Some(current)
}

fn number_at(body: &Map<String, Value>, path: &[&str]) -> Option<f64> {
    match at(body, path)? {
        Value::Number(value) => value.as_f64(),
        Value::String(value) => value.parse::<f64>().ok(),
        _ => None,
    }
}

fn integer_at(body: &Map<String, Value>, path: &[&str]) -> Option<i64> {
    match at(body, path)? {
        Value::Number(value) => value.as_i64(),
        Value::String(value) => value.parse::<i64>().ok(),
        _ => None,
    }
}

fn unsigned_at(body: &Map<String, Value>, path: &[&str]) -> Option<u64> {
    match at(body, path)? {
        Value::Number(value) => value.as_u64(),
        Value::String(value) => value.parse::<u64>().ok(),
        _ => None,
    }
}

fn string_at(body: &Map<String, Value>, path: &[&str]) -> Option<String> {
    match at(body, path)? {
        Value::String(value) if !value.is_empty() => Some(value.clone()),
        _ => None,
    }
}

fn boolean_at(body: &Map<String, Value>, path: &[&str]) -> Option<bool> {
    at(body, path)?.as_bool()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_are_read_by_path_and_never_by_a_body_wide_key_search() {
        let body = r#"{
          "billingCycleStart": "1785200041000",
          "billingCycleEnd": "1787878441000",
          "planUsage": {"limit": 2000, "includedSpend": 1200, "totalPercentUsed": 13.5},
          "spendLimitUsage": {"limit": 999999, "limitType": "user"}
        }"#;
        let mut fields = CursorFields::default();
        read_current_period(&object(body).expect("object"), &mut fields);

        assert_eq!(fields.plan_limit_cents, Some(2000.0));
        assert_eq!(fields.included_spend_cents, Some(1200.0));
        assert_eq!(fields.billing_cycle_end_ms, Some(1787878441000));
    }

    #[test]
    fn the_next_upgrade_price_is_never_read_as_the_current_price() {
        let body = r#"{
          "planInfo": {"planName": "Pro", "price": "$20/mo"},
          "nextUpgrade": {"name": "Pro+", "price": "$60/mo"}
        }"#;
        let mut fields = CursorFields::default();
        read_plan_info(&object(body).expect("object"), &mut fields);

        assert_eq!(fields.plan.as_deref(), Some("Pro"));
        assert_eq!(fields.price.as_ref().map(|price| price.amount), Some(20.0));
    }

    #[test]
    fn an_absent_path_never_falls_back_to_a_same_named_key_elsewhere() {
        let body = r#"{"planUsage": {}, "spendLimitUsage": {"limit": 4242}}"#;
        let mut fields = CursorFields::default();
        read_current_period(&object(body).expect("object"), &mut fields);

        assert_eq!(fields.plan_limit_cents, None);
    }

    #[test]
    fn token_total_is_the_sum_of_all_four_components() {
        let tokens = TokenFacts {
            input: Some(10),
            output: Some(2),
            cache_read: Some(100),
            cache_write: Some(5),
        };

        assert_eq!(tokens.total(), Some(117));
        assert!(!tokens.is_partial());
    }

    #[test]
    fn token_total_stays_unknown_when_a_component_is_missing() {
        let tokens = TokenFacts {
            input: Some(10),
            output: Some(2),
            cache_read: None,
            cache_write: Some(5),
        };

        assert_eq!(tokens.total(), None);
        assert!(tokens.is_partial());
    }

    #[test]
    fn sessions_count_is_the_number_of_distinct_conversations() {
        let mut pages = UsageEventPages::default();
        let more = pages.add_page(
            r#"{"totalUsageEventsCount": 3, "usageEventsDisplay": [
              {"conversationId": "a", "timestamp": "10"},
              {"conversationId": "b", "timestamp": "30"},
              {"conversationId": "a", "timestamp": "20"}
            ]}"#,
        );

        assert!(!more);
        assert!(pages.is_complete());
        let facts = pages.facts();
        assert_eq!(facts.sessions_count, Some(2));
        assert_eq!(facts.events_count, Some(3));
        assert_eq!(facts.latest_activity_ms, Some(30));
        assert!(!facts.sessions_incomplete);
    }

    #[test]
    fn incomplete_paging_reports_no_session_count_instead_of_a_lower_one() {
        let mut pages = UsageEventPages::default();
        pages.add_page(
            r#"{"totalUsageEventsCount": 9, "usageEventsDisplay": [{"conversationId": "a"}]}"#,
        );
        pages.mark_failed();

        let facts = pages.facts();
        assert_eq!(facts.sessions_count, None);
        assert!(facts.sessions_incomplete);
        assert_eq!(facts.events_count, Some(9));
    }

    #[test]
    fn a_page_cap_also_makes_the_session_count_unknown() {
        let mut pages = UsageEventPages::default();
        pages.add_page(
            r#"{"totalUsageEventsCount": 4000, "usageEventsDisplay": [{"conversationId": "a"}]}"#,
        );
        pages.mark_capped();

        assert_eq!(pages.facts().sessions_count, None);
    }

    #[test]
    fn an_event_without_a_conversation_makes_the_session_count_unknown() {
        let mut pages = UsageEventPages::default();
        pages.add_page(
            r#"{"totalUsageEventsCount": 2, "usageEventsDisplay": [{"conversationId": "a"}, {}]}"#,
        );

        assert!(pages.is_complete());
        assert_eq!(pages.facts().sessions_count, None);
    }

    #[test]
    fn identifier_keys_are_dropped_from_raw_data() {
        let value = serde_json::json!({
            "conversationId": "marker",
            "owningUser": "marker",
            "serviceAccountId": "marker",
            "teamId": 77,
            "accessToken": "marker",
            "inputTokens": "123",
            "planName": "Pro"
        });

        let sanitized = sanitize(value).to_string();

        assert!(!sanitized.contains("marker"));
        assert!(!sanitized.contains("77"));
        assert!(sanitized.contains("inputTokens"));
        assert!(sanitized.contains("Pro"));
    }

    #[test]
    fn email_shaped_values_are_dropped_from_raw_data() {
        let value = serde_json::json!({"displayMessage": "billed to person@example.invalid"});

        assert!(!sanitize(value).to_string().contains("example.invalid"));
    }
}
