use crate::types::AccountInfo;

use super::super::common::format_money;
use super::super::time::{format_user_date, TimeContext};

pub fn plan_display_lines(account: &AccountInfo, time_context: &TimeContext) -> Vec<String> {
    let mut lines = Vec::new();

    if let Some(plan) = account.plan.as_deref() {
        if let Some(name) = plan_name_for_display(plan) {
            lines.push(format!("Plan: {name}"));
        }
    }
    if let Some(line) = subscription_dates_line(account, time_context) {
        lines.push(line);
    }
    if let Some(line) = price_line(account) {
        lines.push(line);
    }

    lines
}

fn plan_name_for_display(plan: &str) -> Option<String> {
    let trimmed = plan.trim();
    let mut characters = trimmed.chars();
    let first = characters.next()?;

    Some(first.to_uppercase().chain(characters).collect())
}

fn subscription_dates_line(account: &AccountInfo, time_context: &TimeContext) -> Option<String> {
    let started = account
        .subscription_started_at
        .as_deref()
        .map(|value| format_user_date(value, time_context));
    let renews = account
        .renewal_at
        .as_deref()
        .map(|value| format_user_date(value, time_context));

    match (started, renews) {
        (Some(started), Some(renews)) => Some(format!("Started {started} \u{b7} renews {renews}")),
        (Some(started), None) => Some(format!("Started {started}")),
        (None, Some(renews)) => Some(format!("Renews {renews}")),
        (None, None) => None,
    }
}

fn price_line(account: &AccountInfo) -> Option<String> {
    let amount = account.price_amount?;
    let price = format_money(amount, account.price_currency.as_deref());

    match account.price_note.as_deref().map(str::trim) {
        Some(note) if !note.is_empty() => Some(format!("{price} ({note})")),
        _ => Some(price),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::StructuredSourceInfo;

    fn time_context() -> TimeContext {
        TimeContext::from_structured(&StructuredSourceInfo {
            collected_at: Some("2026-08-02T12:00:00Z".to_string()),
            ..sample_structured()
        })
    }

    fn sample_structured() -> StructuredSourceInfo {
        use crate::types::{SourceStatus, UsageInfo};

        StructuredSourceInfo {
            provider: "codex".to_string(),
            source: "codex_local".to_string(),
            source_link: String::new(),
            status: SourceStatus {
                data_available: true,
                access_available: true,
                message: None,
                cli_authorization: None,
            },
            raw_data_available: true,
            collected_at: None,
            data_as_of: None,
            account: AccountInfo::default(),
            limits: Vec::new(),
            available_limit_resets: None,
            usage: UsageInfo::default(),
            diagnostics: Vec::new(),
        }
    }

    #[test]
    fn empty_account_produces_no_lines() {
        assert!(plan_display_lines(&AccountInfo::default(), &time_context()).is_empty());
    }

    #[test]
    fn credits_alone_produce_no_plan_lines() {
        let account = AccountInfo {
            credits_remaining: Some(38.6355075),
            ..Default::default()
        };

        assert!(plan_display_lines(&account, &time_context()).is_empty());
    }

    #[test]
    fn plan_name_is_capitalized_for_display() {
        let account = AccountInfo {
            plan: Some("plus".to_string()),
            ..Default::default()
        };

        assert_eq!(
            plan_display_lines(&account, &time_context()),
            vec!["Plan: Plus".to_string()]
        );
    }

    #[test]
    fn blank_plan_name_produces_no_line() {
        let account = AccountInfo {
            plan: Some("   ".to_string()),
            ..Default::default()
        };

        assert!(plan_display_lines(&account, &time_context()).is_empty());
    }

    #[test]
    fn both_dates_share_one_line() {
        let account = AccountInfo {
            subscription_started_at: Some("Jan 12, 2026".to_string()),
            renewal_at: Some("Aug 28, 2026".to_string()),
            ..Default::default()
        };
        let lines = plan_display_lines(&account, &time_context());

        assert_eq!(lines.len(), 1);
        assert!(lines[0].starts_with("Started "));
        assert!(lines[0].contains(" \u{b7} renews "));
    }

    #[test]
    fn started_alone_renders_its_own_line() {
        let account = AccountInfo {
            subscription_started_at: Some("Jan 12, 2026".to_string()),
            ..Default::default()
        };
        let lines = plan_display_lines(&account, &time_context());

        assert_eq!(lines.len(), 1);
        assert!(lines[0].starts_with("Started "));
        assert!(!lines[0].contains("renews"));
    }

    #[test]
    fn renewal_alone_renders_its_own_line() {
        let account = AccountInfo {
            renewal_at: Some("Aug 28, 2026".to_string()),
            ..Default::default()
        };
        let lines = plan_display_lines(&account, &time_context());

        assert_eq!(lines.len(), 1);
        assert!(lines[0].starts_with("Renews "));
    }

    #[test]
    fn iso_dates_render_as_dates_without_a_time_component() {
        let context = time_context();
        let account = AccountInfo {
            renewal_at: Some("2026-08-28T12:00:00Z".to_string()),
            ..Default::default()
        };
        let lines = plan_display_lines(&account, &context);

        assert_eq!(
            lines,
            vec![format!(
                "Renews {}",
                format_user_date("2026-08-28T12:00:00Z", &context)
            )]
        );
        assert!(!lines[0].contains("2026-08-28T"));
        assert!(!lines[0].contains(':'));
    }

    #[test]
    fn subscription_dates_always_carry_their_year() {
        let context = time_context();
        let account = AccountInfo {
            subscription_started_at: Some("2024-01-12T12:00:00Z".to_string()),
            renewal_at: Some("2027-01-12T12:00:00Z".to_string()),
            ..Default::default()
        };
        let lines = plan_display_lines(&account, &context);

        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("2024"));
        assert!(lines[0].contains("2027"));
        assert!(!lines[0].contains(':'));
    }

    #[test]
    fn price_note_is_appended_in_parentheses() {
        let account = AccountInfo {
            price_amount: Some(20.0),
            price_currency: Some("USD".to_string()),
            price_note: Some("may vary by country/currency".to_string()),
            ..Default::default()
        };

        assert_eq!(
            plan_display_lines(&account, &time_context()),
            vec!["$20.00 (may vary by country/currency)".to_string()]
        );
    }

    #[test]
    fn price_without_note_renders_alone() {
        let account = AccountInfo {
            price_amount: Some(20.0),
            price_currency: Some("USD".to_string()),
            ..Default::default()
        };

        assert_eq!(
            plan_display_lines(&account, &time_context()),
            vec!["$20.00".to_string()]
        );
    }

    #[test]
    fn non_usd_price_keeps_its_currency_code() {
        let account = AccountInfo {
            price_amount: Some(18.5),
            price_currency: Some("EUR".to_string()),
            ..Default::default()
        };

        assert_eq!(
            plan_display_lines(&account, &time_context()),
            vec!["18.50 EUR".to_string()]
        );
    }

    #[test]
    fn price_note_without_amount_produces_no_line() {
        let account = AccountInfo {
            price_note: Some("may vary by country/currency".to_string()),
            price_currency: Some("USD".to_string()),
            ..Default::default()
        };

        assert!(plan_display_lines(&account, &time_context()).is_empty());
    }

    #[test]
    fn management_urls_produce_no_lines() {
        let account = AccountInfo {
            plan_management_url: Some("https://example.test/plan".to_string()),
            billing_management_url: Some("https://example.test/billing".to_string()),
            ..Default::default()
        };

        assert!(plan_display_lines(&account, &time_context()).is_empty());
    }

    #[test]
    fn full_account_renders_every_line_in_order() {
        let account = AccountInfo {
            plan: Some("pro".to_string()),
            subscription_started_at: Some("Jan 12, 2026".to_string()),
            renewal_at: Some("Aug 28, 2026".to_string()),
            price_amount: Some(20.0),
            price_currency: Some("usd".to_string()),
            price_note: Some("may vary by country/currency".to_string()),
            ..Default::default()
        };
        let lines = plan_display_lines(&account, &time_context());

        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "Plan: Pro");
        assert!(lines[1].starts_with("Started "));
        assert_eq!(lines[2], "$20.00 (may vary by country/currency)");
    }
}
