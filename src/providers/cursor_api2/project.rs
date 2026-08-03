use crate::types::{
    AccountInfo, ActivityUsage, LimitInfo, MoneyUsage, SourceData, SourceStatus,
    StructuredSourceInfo, TokenUsage, UsageInfo,
};

use super::fetch::CursorResponses;
use super::helpers::{
    amount_percents, billing_cycle_label, cents_to_usd, complementary_percent, fill_amount_triple,
    format_unix_ms_timestamp, percent_limit, utc_now,
};
use super::parse::{assemble, sanitized_raw, CursorFields};

const PROVIDER: &str = "cursor";
const SOURCE: &str = "cursor_api2";
const SOURCE_LINK: &str = "docs/get-limits/providers/cursor.md";

const CURRENCY: &str = "USD";
const AMOUNT_UNIT: &str = "usd";

/// Cursor does not state that the price it reports is what every user on the
/// plan pays, so the disclaimer rule applies. The note is a fixed literal and
/// never interpolates response content.
const PRICE_NOTE: &str =
    "price as reported for this account; the amount can differ by country, currency, tax, and promotion";

const UNRECOGNIZED: &str = "Cursor api2 usage unavailable: response format is not recognized";

const HARD_LIMIT_SCALE_NOTE: &str =
    "on-demand spend window: the amount scale reported by the hard limit method is unverified";

pub fn build_source_data(responses: &CursorResponses) -> SourceData {
    let collected_at = utc_now();
    let fields = assemble(responses);
    let raw = sanitized_raw(responses);

    if fields.is_empty() {
        return SourceData {
            structured: StructuredSourceInfo {
                provider: PROVIDER.to_string(),
                source: SOURCE.to_string(),
                source_link: SOURCE_LINK.to_string(),
                status: SourceStatus {
                    data_available: false,
                    access_available: true,
                    message: Some(UNRECOGNIZED.to_string()),
                    cli_authorization: None,
                },
                raw_data_available: raw.is_some(),
                collected_at: Some(collected_at),
                data_as_of: None,
                account: AccountInfo::default(),
                limits: Vec::new(),
                available_limit_resets: None,
                usage: UsageInfo::default(),
                diagnostics: fields.diagnostics,
            },
            raw,
            stderr: String::new(),
        };
    }

    let mut diagnostics = fields.diagnostics.clone();
    let limits = build_limits(&fields, &mut diagnostics);

    SourceData {
        structured: StructuredSourceInfo {
            provider: PROVIDER.to_string(),
            source: SOURCE.to_string(),
            source_link: SOURCE_LINK.to_string(),
            status: SourceStatus {
                data_available: true,
                access_available: true,
                message: None,
                cli_authorization: None,
            },
            raw_data_available: raw.is_some(),
            collected_at: Some(collected_at.clone()),
            data_as_of: Some(collected_at),
            account: build_account(&fields),
            limits,
            available_limit_resets: None,
            usage: build_usage(&fields),
            diagnostics,
        },
        raw,
        stderr: String::new(),
    }
}

fn build_account(fields: &CursorFields) -> AccountInfo {
    AccountInfo {
        plan: fields.plan.clone(),
        price_amount: fields.price.as_ref().map(|price| price.amount),
        price_currency: fields.price.as_ref().map(|price| price.currency.clone()),
        price_period: fields.price.as_ref().map(|price| price.period.clone()),
        price_note: fields.price.as_ref().map(|_| PRICE_NOTE.to_string()),
        renewal_at: fields.renewal_at_ms.map(format_unix_ms_timestamp),
        // Confirmed source limits: no method reports when the plan began, and
        // no account-specific portal link is issued for an individual account.
        subscription_started_at: None,
        plan_management_url: None,
        billing_management_url: None,
        ..AccountInfo::default()
    }
}

fn build_limits(fields: &CursorFields, diagnostics: &mut Vec<String>) -> Vec<LimitInfo> {
    let resets_at = fields.billing_cycle_end_ms.map(format_unix_ms_timestamp);
    let window_label =
        billing_cycle_label(fields.billing_cycle_start_ms, fields.billing_cycle_end_ms);
    let mut limits = Vec::new();

    if let Some(used_percent) = fields.total_percent_used {
        limits.push(LimitInfo {
            name: "plan_usage".to_string(),
            window_label: window_label.clone(),
            window_minutes: None,
            resets_at: resets_at.clone(),
            used_percent: Some(used_percent),
            remaining_percent: Some(complementary_percent(used_percent)),
            ..LimitInfo::default()
        });
    }

    if let Some(used_percent) = fields.auto_percent_used {
        limits.push(percent_limit("auto", used_percent, resets_at.clone()));
    }

    if let Some(used_percent) = fields.api_percent_used {
        limits.push(percent_limit("api_models", used_percent, resets_at.clone()));
    }

    if fields.included_spend_cents.is_some() || fields.plan_limit_cents.is_some() {
        let (used, remaining, total) = included_spend(fields);
        let (used_percent, remaining_percent) = amount_percents(used, total);

        limits.push(LimitInfo {
            name: "included_spend".to_string(),
            window_label,
            window_minutes: None,
            resets_at: resets_at.clone(),
            used_percent,
            remaining_percent,
            used_amount: used,
            remaining_amount: remaining,
            total_amount: total,
            amount_unit: Some(AMOUNT_UNIT.to_string()),
        });
    }

    // A ceiling only exists when usage-based spend is permitted at all; when it
    // is not, there is no on-demand window to show.
    if fields.usage_based_allowed == Some(true) {
        if let Some(hard_limit) = fields.hard_limit {
            let (used, remaining, total) =
                fill_amount_triple(fields.individual_used, None, Some(hard_limit));
            let (used_percent, remaining_percent) = amount_percents(used, total);

            limits.push(LimitInfo {
                name: "on_demand_spend".to_string(),
                window_label: None,
                window_minutes: None,
                resets_at,
                used_percent,
                remaining_percent,
                used_amount: used,
                remaining_amount: remaining,
                total_amount: total,
                amount_unit: Some(AMOUNT_UNIT.to_string()),
            });
            diagnostics.push(HARD_LIMIT_SCALE_NOTE.to_string());
        }
    }

    limits
}

fn included_spend(fields: &CursorFields) -> (Option<f64>, Option<f64>, Option<f64>) {
    fill_amount_triple(
        cents_to_usd(fields.included_spend_cents),
        None,
        cents_to_usd(fields.plan_limit_cents),
    )
}

fn build_usage(fields: &CursorFields) -> UsageInfo {
    let (used, remaining, total) = included_spend(fields);
    let tokens = fields.tokens;

    UsageInfo {
        tokens: TokenUsage {
            input: tokens.input,
            output: tokens.output,
            cache_read: tokens.cache_read,
            cache_write: tokens.cache_write,
            total: tokens.total(),
            // The response carries no such breakdown, and `cache_read` is not
            // the same fact as `cached_input`.
            cached_input: None,
            reasoning_output: None,
        },
        money: MoneyUsage {
            used_amount: used,
            remaining_amount: remaining,
            total_amount: total,
            currency: total.map(|_| CURRENCY.to_string()),
        },
        activity: ActivityUsage {
            events_count: fields.activity.events_count,
            // `totalUsageEventsCount` counts billable usage events; it is the
            // closest honest match the source has for a turn count.
            turns_count: fields.activity.events_count,
            sessions_count: fields.activity.sessions_count,
            latest_activity_at: fields
                .activity
                .latest_activity_ms
                .map(format_unix_ms_timestamp),
            // Confirmed source limit: no usage message carries a file counter.
            files_count: None,
        },
        // `aggregations[].modelIntent` groups events by intent, and it is not
        // established that its value is a model name.
        models: Default::default(),
    }
}

pub(super) fn access_denied(message: String) -> SourceData {
    SourceData {
        raw: None,
        structured: StructuredSourceInfo {
            provider: PROVIDER.to_string(),
            source: SOURCE.to_string(),
            source_link: SOURCE_LINK.to_string(),
            status: SourceStatus {
                data_available: false,
                access_available: false,
                message: Some(message),
                cli_authorization: None,
            },
            raw_data_available: false,
            collected_at: Some(utc_now()),
            data_as_of: None,
            account: AccountInfo::default(),
            limits: Vec::new(),
            available_limit_resets: None,
            usage: UsageInfo::default(),
            diagnostics: Vec::new(),
        },
        stderr: String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::cursor_api2::parse::UsageEventPages;

    const PLAN_INFO: &str = r#"{
      "planInfo": {
        "planName": "Pro",
        "includedAmountCents": 2000,
        "price": "$20/mo",
        "billingCycleEnd": "1787878441000",
        "planOwner": "PLAN_OWNER_STRIPE"
      },
      "nextUpgrade": {"tier": "pro_plus", "name": "Pro+", "includedAmountCents": 7000, "price": "$60/mo"}
    }"#;

    const CURRENT_PERIOD: &str = r#"{
      "billingCycleStart": "1785200041000",
      "billingCycleEnd": "1787878441000",
      "planUsage": {
        "totalSpend": 4690,
        "includedSpend": 500,
        "limit": 2000,
        "autoPercentUsed": 14.85,
        "apiPercentUsed": 5.17,
        "totalPercentUsed": 13.59
      },
      "spendLimitUsage": {"limitType": "user"},
      "displayMessage": "You've hit your usage limit"
    }"#;

    const HARD_LIMIT: &str = r#"{"noUsageBasedAllowed": true}"#;

    const AGGREGATED: &str = r#"{
      "aggregations": [{"modelIntent": "default", "inputTokens": "10", "totalCents": 3049.27, "tier": 2}],
      "totalInputTokens": "9573974",
      "totalOutputTokens": "1156821",
      "totalCacheWriteTokens": "411287",
      "totalCacheReadTokens": "77072093",
      "totalCostCents": 4690.76
    }"#;

    fn events_page(events: &str, total: u64) -> String {
        format!(r#"{{"totalUsageEventsCount": {total}, "usageEventsDisplay": [{events}]}}"#)
    }

    fn responses() -> CursorResponses {
        let mut filtered = UsageEventPages::default();
        filtered.add_page(&events_page(
            r#"{"timestamp": "1785707242358", "model": "default", "conversationId": "one", "owningUser": "someone", "serviceAccountId": "sa"},
               {"timestamp": "1785707242000", "model": "default", "conversationId": "two"},
               {"timestamp": "1785707241000", "model": "default", "conversationId": "one"}"#,
            3,
        ));

        CursorResponses {
            plan_info: Some(Ok(PLAN_INFO.to_string())),
            current_period: Some(Ok(CURRENT_PERIOD.to_string())),
            hard_limit: Some(Ok(HARD_LIMIT.to_string())),
            aggregated: Some(Ok(AGGREGATED.to_string())),
            filtered,
            fetch_diagnostics: Vec::new(),
        }
    }

    fn limit<'a>(data: &'a SourceData, name: &str) -> &'a LimitInfo {
        data.structured
            .limits
            .iter()
            .find(|limit| limit.name == name)
            .unwrap_or_else(|| panic!("{name} limit"))
    }

    #[test]
    fn projects_the_full_response_set() {
        let data = build_source_data(&responses());
        let structured = &data.structured;

        assert!(structured.status.access_available);
        assert!(structured.status.data_available);
        assert_eq!(structured.status.message, None);
        assert!(structured.raw_data_available);
        assert_eq!(structured.data_as_of, structured.collected_at);

        assert_eq!(structured.account.plan.as_deref(), Some("Pro"));
        assert_eq!(structured.account.price_amount, Some(20.0));
        assert_eq!(structured.account.price_currency.as_deref(), Some("USD"));
        assert_eq!(structured.account.price_period.as_deref(), Some("mo"));
        assert_eq!(structured.account.price_note.as_deref(), Some(PRICE_NOTE));
        assert_eq!(
            structured.account.renewal_at.as_deref(),
            Some("2026-08-28T00:54:01Z")
        );
        assert_eq!(structured.account.subscription_started_at, None);
        assert_eq!(structured.account.plan_management_url, None);
        assert_eq!(structured.account.billing_management_url, None);

        let plan = limit(&data, "plan_usage");
        assert_eq!(plan.used_percent, Some(13.59));
        assert_eq!(plan.remaining_percent, Some(86.41));
        assert_eq!(plan.resets_at.as_deref(), Some("2026-08-28T00:54:01Z"));
        assert_eq!(
            plan.window_label.as_deref(),
            Some("2026-07-28 -> 2026-08-28")
        );

        let spend = limit(&data, "included_spend");
        assert_eq!(spend.used_amount, Some(5.0));
        assert_eq!(spend.remaining_amount, Some(15.0));
        assert_eq!(spend.total_amount, Some(20.0));
        assert_eq!(spend.used_percent, Some(25.0));
        assert_eq!(spend.amount_unit.as_deref(), Some("usd"));

        assert_eq!(limit(&data, "auto").used_percent, Some(14.85));
        assert_eq!(limit(&data, "api_models").used_percent, Some(5.17));

        let tokens = &structured.usage.tokens;
        assert_eq!(tokens.input, Some(9_573_974));
        assert_eq!(tokens.output, Some(1_156_821));
        assert_eq!(tokens.cache_write, Some(411_287));
        assert_eq!(tokens.cache_read, Some(77_072_093));
        assert_eq!(tokens.total, Some(88_214_175));
        assert_eq!(tokens.cached_input, None);
        assert_eq!(tokens.reasoning_output, None);

        let activity = &structured.usage.activity;
        assert_eq!(activity.events_count, Some(3));
        assert_eq!(activity.turns_count, Some(3));
        assert_eq!(activity.sessions_count, Some(2));
        assert_eq!(
            activity.latest_activity_at.as_deref(),
            Some("2026-08-02T21:47:22Z")
        );
        assert_eq!(activity.files_count, None);
        assert_eq!(structured.usage.models.top_model, None);
    }

    #[test]
    fn no_on_demand_window_when_usage_based_spend_is_not_allowed() {
        let data = build_source_data(&responses());

        assert!(!data
            .structured
            .limits
            .iter()
            .any(|limit| limit.name == "on_demand_spend"));
    }

    #[test]
    fn projects_the_on_demand_window_when_usage_based_spend_is_allowed() {
        let mut responses = responses();
        responses.hard_limit = Some(Ok(
            r#"{"noUsageBasedAllowed": false, "hardLimit": 50}"#.to_string()
        ));
        responses.current_period = Some(Ok(CURRENT_PERIOD.replace(
            r#""spendLimitUsage": {"limitType": "user"}"#,
            r#""spendLimitUsage": {"individualUsed": 10, "limitType": "user"}"#,
        )));

        let data = build_source_data(&responses);
        let on_demand = limit(&data, "on_demand_spend");

        assert_eq!(on_demand.used_amount, Some(10.0));
        assert_eq!(on_demand.total_amount, Some(50.0));
        assert_eq!(on_demand.remaining_amount, Some(40.0));
        assert_eq!(on_demand.used_percent, Some(20.0));
        assert!(data
            .structured
            .diagnostics
            .iter()
            .any(|entry| entry == HARD_LIMIT_SCALE_NOTE));
    }

    #[test]
    fn an_unrecognized_price_leaves_all_three_price_fields_null() {
        for price in ["$20", "20/mo", "from $20/mo", "$20/month"] {
            let mut responses = responses();
            responses.plan_info = Some(Ok(PLAN_INFO.replace("$20/mo", price)));

            let structured = build_source_data(&responses).structured;

            assert_eq!(structured.account.price_amount, None, "{price}");
            assert_eq!(structured.account.price_currency, None, "{price}");
            assert_eq!(structured.account.price_period, None, "{price}");
            assert_eq!(structured.account.price_note, None, "{price}");
            assert!(
                structured
                    .diagnostics
                    .iter()
                    .any(|entry| entry.starts_with("price:")),
                "{price} should be reported in diagnostics"
            );
            // The plan itself still projects; a price failure degrades only the price.
            assert_eq!(structured.account.plan.as_deref(), Some("Pro"));
        }
    }

    #[test]
    fn a_missing_token_component_leaves_the_total_null() {
        let mut responses = responses();
        responses.aggregated = Some(Ok(AGGREGATED
            .replace(r#""totalCacheWriteTokens": "411287","#, "")
            .to_string()));

        let structured = build_source_data(&responses).structured;

        assert_eq!(structured.usage.tokens.cache_write, None);
        assert_eq!(structured.usage.tokens.input, Some(9_573_974));
        assert_eq!(structured.usage.tokens.total, None);
        assert!(structured
            .diagnostics
            .iter()
            .any(|entry| entry.starts_with("total tokens:")));
    }

    #[test]
    fn incomplete_paging_leaves_the_session_count_null() {
        let mut responses = responses();
        let mut filtered = UsageEventPages::default();
        filtered.add_page(&events_page(r#"{"conversationId": "one"}"#, 4000));
        filtered.mark_failed();
        responses.filtered = filtered;

        let structured = build_source_data(&responses).structured;

        assert_eq!(structured.usage.activity.sessions_count, None);
        assert_eq!(structured.usage.activity.turns_count, Some(4000));
        assert!(structured
            .diagnostics
            .iter()
            .any(|entry| entry.starts_with("sessions:")));
    }

    #[test]
    fn a_failed_step_degrades_only_its_own_fields() {
        let mut responses = responses();
        responses.aggregated = Some(Err("request failed"));
        responses.filtered = UsageEventPages::default();

        let data = build_source_data(&responses);

        assert!(data.structured.status.data_available);
        assert_eq!(data.structured.account.plan.as_deref(), Some("Pro"));
        assert_eq!(data.structured.usage.tokens.total, None);
        assert_eq!(data.structured.usage.activity.sessions_count, None);
        assert_eq!(limit(&data, "plan_usage").used_percent, Some(13.59));
    }

    #[test]
    fn marks_an_unrecognized_response_set_as_accessible_without_data() {
        let responses = CursorResponses {
            plan_info: Some(Ok(r#"{"unexpected": "shape"}"#.to_string())),
            ..CursorResponses::default()
        };

        let structured = build_source_data(&responses).structured;

        assert!(structured.status.access_available);
        assert!(!structured.status.data_available);
        assert_eq!(structured.status.message.as_deref(), Some(UNRECOGNIZED));
    }

    #[test]
    fn represents_access_denied_without_raw_data() {
        let data = access_denied(
            "Cursor api2 usage unavailable: token not found; run `cursor agent login`".to_string(),
        );

        assert!(!data.structured.status.access_available);
        assert!(!data.structured.status.data_available);
        assert!(!data.structured.raw_data_available);
        assert!(data.raw.is_none());
        assert!(data.structured.limits.is_empty());
    }

    /// No account identifier may reach any published artifact. The markers are
    /// placed in every field the responses are documented to carry them in.
    #[test]
    fn no_account_identifier_reaches_any_published_artifact() {
        const MARKERS: &[&str] = &[
            "TOKENMARKER0000000000",
            "marker.person@example.invalid",
            "USERIDMARKER",
            "AUTHIDMARKER",
            "WORKOSIDMARKER",
            "TEAMIDMARKER",
            "CONVERSATIONMARKER",
            "SERVICEACCOUNTMARKER",
            "OWNINGUSERMARKER",
            "SUBSCRIPTIONPRODUCTMARKER",
        ];

        let mut filtered = UsageEventPages::default();
        filtered.add_page(&events_page(
            r#"{"timestamp": "1785707242358", "conversationId": "CONVERSATIONMARKER-1", "owningUser": "OWNINGUSERMARKER", "serviceAccountId": "SERVICEACCOUNTMARKER", "subscriptionProductId": "SUBSCRIPTIONPRODUCTMARKER"},
               {"timestamp": "1785707242000", "conversationId": "CONVERSATIONMARKER-2", "owningUser": "marker.person@example.invalid"}"#,
            2,
        ));

        let responses = CursorResponses {
            plan_info: Some(Ok(PLAN_INFO.replace(
                r#""planOwner": "PLAN_OWNER_STRIPE""#,
                r#""planOwner": "PLAN_OWNER_STRIPE", "accessToken": "TOKENMARKER0000000000", "userId": "USERIDMARKER""#,
            ))),
            current_period: Some(Ok(CURRENT_PERIOD.replace(
                r#""displayMessage": "You've hit your usage limit""#,
                r#""displayMessage": "marker.person@example.invalid hit the usage limit", "teamId": "TEAMIDMARKER", "authId": "AUTHIDMARKER", "workosId": "WORKOSIDMARKER""#,
            ))),
            hard_limit: Some(Ok(HARD_LIMIT.to_string())),
            aggregated: Some(Ok(AGGREGATED.to_string())),
            filtered,
            fetch_diagnostics: Vec::new(),
        };

        let data = build_source_data(&responses);
        let structured = &data.structured;
        let artifacts = [
            data.raw.clone().unwrap_or_default(),
            data.stderr.clone(),
            serde_json::to_string(structured).expect("structured data serializes"),
            structured.status.message.clone().unwrap_or_default(),
            structured.diagnostics.join("\n"),
        ];

        for marker in MARKERS {
            for artifact in &artifacts {
                assert!(
                    !artifact.contains(marker),
                    "{marker} leaked into a published artifact"
                );
            }
        }

        // The run still produced data, so the assertions above are not vacuous.
        assert!(data.raw.is_some());
        assert!(structured.status.data_available);
        assert_eq!(structured.usage.activity.sessions_count, Some(2));
    }
}
