use chrono::{DateTime, Utc};

use crate::types::{
    AccountInfo, LimitInfo, MoneyUsage, SourceData, SourceStatus, StructuredSourceInfo, UsageInfo,
};

use super::parse::{
    format_iso_utc, format_utc, plan_name, remaining_percent, scale_minor_units,
    ClaudeUsageResponse, ExtraUsage, LimitRecord, RateLimitWindow, Spend, CRITICAL_SEVERITY,
    FIVE_HOUR_LIMIT_KIND, FIVE_HOUR_WINDOW, FIVE_HOUR_WINDOW_MINUTES, NORMAL_SEVERITY,
    SEVEN_DAY_LIMIT_KIND, SEVEN_DAY_WINDOW, SEVEN_DAY_WINDOW_MINUTES, USD_CURRENCY,
};

const PROVIDER: &str = "claude";
const SOURCE: &str = "claude_rpc";
const SOURCE_LINK: &str = "https://code.claude.com/docs/en/setup";

pub(super) const CLI_MISSING_MESSAGE: &str = "Claude CLI is not installed or is not available in PATH; install `claude` and try again. Setup: https://code.claude.com/docs/en/setup";
const NO_DATA_MESSAGE: &str = "no supported limit or usage data in the Claude CLI usage response";
const RATE_LIMITS_UNAVAILABLE_MESSAGE: &str =
    "the Claude CLI reports that rate limits are not available for this account";

/// The unit the server's own field names state. No conversion is ever applied
/// to make it agree with the account's billing currency.
const DOLLAR_AMOUNT_UNIT: &str = "usd";

pub(super) fn build_source_data(
    response: Option<&ClaudeUsageResponse>,
    collected_at: DateTime<Utc>,
    data_as_of: DateTime<Utc>,
) -> SourceData {
    let structured = build_structured(response, collected_at, data_as_of);
    let raw = response.and_then(|response| serde_json::to_string_pretty(response).ok());

    SourceData {
        raw,
        structured,
        stderr: String::new(),
    }
}

/// Projects the response per the table in
/// [claude-rpc-usage.md](../../../docs/get-limits/providers/claude-rpc-usage.md).
/// Every value that cannot be read as reported degrades to `null` plus a short
/// fixed diagnostic; nothing is guessed and no response content is quoted.
fn build_structured(
    response: Option<&ClaudeUsageResponse>,
    collected_at: DateTime<Utc>,
    data_as_of: DateTime<Utc>,
) -> StructuredSourceInfo {
    let mut diagnostics = Vec::new();
    let mut account = AccountInfo::default();
    let mut limits = Vec::new();
    let mut money = MoneyUsage::default();
    let mut rate_limits_available = true;

    let Some(response) = response else {
        diagnostics.push("usage: the response could not be read".to_string());
        return no_data_info(collected_at, diagnostics, NO_DATA_MESSAGE);
    };

    account.plan = project_plan(response.subscription_type.as_deref(), &mut diagnostics);

    if response.rate_limits_available == Some(false) {
        rate_limits_available = false;
        diagnostics.push("limits: the account reports no available rate limits".to_string());
    }

    if let Some(rate_limits) = response.rate_limits.as_ref() {
        limits.extend(project_limit(
            FIVE_HOUR_WINDOW,
            FIVE_HOUR_WINDOW_MINUTES,
            rate_limits.five_hour.as_ref(),
            response.limit_record(FIVE_HOUR_LIMIT_KIND),
            &mut diagnostics,
        ));
        limits.extend(project_limit(
            SEVEN_DAY_WINDOW,
            SEVEN_DAY_WINDOW_MINUTES,
            rate_limits.seven_day.as_ref(),
            response.limit_record(SEVEN_DAY_LIMIT_KIND),
            &mut diagnostics,
        ));

        if let Some(extra_usage) = rate_limits.extra_usage.as_ref() {
            project_credits(extra_usage, &mut account, &mut diagnostics);
        }
        if let Some(spend) = rate_limits.spend.as_ref() {
            money = project_money(spend, &mut diagnostics);
        }
    }

    if response.behaviors.is_some() {
        diagnostics.push(
            "activity: the reported counts are a windowed local scan of this machine only and are not account totals".to_string(),
        );
    }

    let data_available = !limits.is_empty()
        || account.plan.is_some()
        || account.credits_used.is_some()
        || account.credits_total.is_some()
        || money.used_amount.is_some();

    let message = if !rate_limits_available && limits.is_empty() {
        Some(RATE_LIMITS_UNAVAILABLE_MESSAGE.to_string())
    } else {
        (!data_available).then(|| NO_DATA_MESSAGE.to_string())
    };

    StructuredSourceInfo {
        provider: PROVIDER.to_string(),
        source: SOURCE.to_string(),
        source_link: SOURCE_LINK.to_string(),
        status: SourceStatus {
            data_available: data_available && rate_limits_available,
            access_available: true,
            message,
            cli_authorization: None,
        },
        raw_data_available: true,
        collected_at: Some(format_utc(collected_at)),
        data_as_of: data_available.then(|| format_utc(data_as_of)),
        account,
        limits,
        available_limit_resets: None,
        // `usage.tokens`, `usage.activity`, and `usage.models` stay empty:
        // the only token data is the collector's own process, and the activity
        // counts are a windowed single-machine estimate. Both come from
        // `claude_local` for this provider.
        usage: UsageInfo {
            money,
            ..UsageInfo::default()
        },
        diagnostics,
    }
}

pub(super) fn unavailable_source_data(message: &str) -> SourceData {
    SourceData {
        raw: None,
        structured: StructuredSourceInfo {
            provider: PROVIDER.to_string(),
            source: SOURCE.to_string(),
            source_link: SOURCE_LINK.to_string(),
            status: SourceStatus {
                data_available: false,
                access_available: false,
                message: Some(message.to_string()),
                cli_authorization: None,
            },
            raw_data_available: false,
            collected_at: Some(format_utc(Utc::now())),
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

fn no_data_info(
    collected_at: DateTime<Utc>,
    diagnostics: Vec<String>,
    message: &str,
) -> StructuredSourceInfo {
    StructuredSourceInfo {
        provider: PROVIDER.to_string(),
        source: SOURCE.to_string(),
        source_link: SOURCE_LINK.to_string(),
        status: SourceStatus {
            data_available: false,
            access_available: true,
            message: Some(message.to_string()),
            cli_authorization: None,
        },
        raw_data_available: false,
        collected_at: Some(format_utc(collected_at)),
        data_as_of: None,
        account: AccountInfo::default(),
        limits: Vec::new(),
        available_limit_resets: None,
        usage: UsageInfo::default(),
        diagnostics,
    }
}

/// A missing `subscription_type` is a normal value for API-key, Bedrock, and
/// Vertex accounts, not a parse failure.
fn project_plan(subscription_type: Option<&str>, diagnostics: &mut Vec<String>) -> Option<String> {
    let subscription_type = subscription_type?;
    let plan = plan_name(subscription_type).map(str::to_string);
    if plan.is_none() {
        diagnostics
            .push("account plan: the reported subscription tier is not recognized".to_string());
    }
    plan
}

/// One record per named window. The named window is authoritative for amounts
/// and `utilization`; the matching `rate_limits.limits[]` entry fills in the
/// percent and reset time when the window omits them, and its `severity` and
/// `is_active` reach diagnostics only. The two never produce two records.
fn project_limit(
    name: &str,
    window_minutes: u64,
    window: Option<&RateLimitWindow>,
    record: Option<&LimitRecord>,
    diagnostics: &mut Vec<String>,
) -> Option<LimitInfo> {
    if window.is_none() && record.is_none() {
        return None;
    }

    let used_percent = window
        .and_then(|window| window.utilization)
        .or_else(|| record.and_then(|record| record.percent));
    let resets_at = window
        .and_then(|window| window.resets_at.as_deref())
        .or_else(|| record.and_then(|record| record.resets_at.as_deref()))
        .and_then(format_iso_utc);
    let used_amount = window.and_then(|window| window.used_dollars);
    let total_amount = window.and_then(|window| window.limit_dollars);
    let remaining_amount = window.and_then(|window| window.remaining_dollars);
    let has_amount = used_amount.is_some() || total_amount.is_some() || remaining_amount.is_some();

    if used_percent.is_none() && resets_at.is_none() && !has_amount {
        return None;
    }

    if let Some(note) = severity_note(name, record) {
        diagnostics.push(note.to_string());
    }

    Some(LimitInfo {
        name: name.to_string(),
        window_label: Some(name.to_string()),
        window_minutes: Some(window_minutes),
        resets_at,
        used_percent,
        remaining_percent: used_percent.map(remaining_percent),
        used_amount,
        remaining_amount,
        total_amount,
        amount_unit: has_amount.then(|| DOLLAR_AMOUNT_UNIT.to_string()),
    })
}

/// `severity` and `is_active` have no structured field. They are mapped to
/// fixed literals so that no response content is interpolated.
fn severity_note(name: &str, record: Option<&LimitRecord>) -> Option<&'static str> {
    let record = record?;
    match record.severity.as_deref()? {
        NORMAL_SEVERITY => None,
        CRITICAL_SEVERITY => match (name, record.is_active) {
            (FIVE_HOUR_WINDOW, Some(true)) => {
                Some("limits: the 5-hour window is active and reported as critical")
            }
            (FIVE_HOUR_WINDOW, _) => Some("limits: the 5-hour window is reported as critical"),
            (SEVEN_DAY_WINDOW, Some(true)) => {
                Some("limits: the 7-day window is active and reported as critical")
            }
            _ => Some("limits: the 7-day window is reported as critical"),
        },
        _ => Some("limits: the reported limit severity is not recognized"),
    }
}

/// The extra-usage allowance is monetary for Claude. A disabled allowance is
/// not an available balance, so all three credit fields stay `null` and the
/// diagnostic carries nothing but the disabled state.
fn project_credits(
    extra_usage: &ExtraUsage,
    account: &mut AccountInfo,
    diagnostics: &mut Vec<String>,
) {
    if extra_usage.is_enabled != Some(true) {
        diagnostics.push("credits: the extra usage allowance is disabled".to_string());
        return;
    }

    let Some(decimal_places) = extra_usage.decimal_places else {
        if extra_usage.monthly_limit.is_some() || extra_usage.used_credits.is_some() {
            diagnostics
                .push("credits: the reported amount scale is missing or unusable".to_string());
        }
        return;
    };

    let scaled =
        |amount: Option<i64>| amount.and_then(|amount| scale_minor_units(amount, decimal_places));
    account.credits_total = scaled(extra_usage.monthly_limit);
    account.credits_used = scaled(extra_usage.used_credits);

    if (extra_usage.monthly_limit.is_some() && account.credits_total.is_none())
        || (extra_usage.used_credits.is_some() && account.credits_used.is_none())
    {
        diagnostics.push("credits: the reported amount scale is missing or unusable".to_string());
    }

    account.credits_remaining = match (account.credits_total, account.credits_used) {
        (Some(total), Some(used)) => Some(total - used),
        _ => None,
    };
}

/// The account's spend, in the currency the account is billed in. The limit
/// amounts are named `*_dollars` by the server; when the two disagree the
/// difference is recorded and neither value is re-labelled or converted.
fn project_money(spend: &Spend, diagnostics: &mut Vec<String>) -> MoneyUsage {
    let used = spend.used.as_ref();
    let exponent = used.and_then(|used| used.exponent);
    let currency = used.and_then(|used| used.currency.clone());

    let scaled = |amount: Option<i64>| match (amount, exponent) {
        (Some(amount), Some(exponent)) => scale_minor_units(amount, exponent),
        _ => None,
    };
    let used_amount = scaled(used.and_then(|used| used.amount_minor));
    let total_amount = scaled(spend.limit);

    if used.and_then(|used| used.amount_minor).is_some() && used_amount.is_none() {
        diagnostics.push("spend: the reported amount scale is missing or unusable".to_string());
    }
    if currency
        .as_deref()
        .is_some_and(|currency| !currency.eq_ignore_ascii_case(USD_CURRENCY))
    {
        diagnostics.push(
            "spend: the account is billed in a currency other than US dollars while the limit amounts are reported as dollars; no conversion is applied".to_string(),
        );
    }

    MoneyUsage {
        used_amount,
        total_amount,
        remaining_amount: match (total_amount, used_amount) {
            (Some(total), Some(used)) => Some(total - used),
            _ => None,
        },
        currency,
    }
}
