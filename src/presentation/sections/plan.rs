use crate::types::AccountInfo;

use super::super::common::format_money;
use super::super::time::{format_user_date, TimeContext};

pub fn plan_display_lines(account: &AccountInfo, time_context: &TimeContext) -> Vec<String> {
    let mut lines = Vec::new();

    if let Some(line) = plan_and_price_line(account) {
        lines.push(line);
    }
    if let Some(renewal) = account.renewal_at.as_deref() {
        lines.push(format!(
            "renews {}",
            format_user_date(renewal, time_context)
        ));
    }

    lines
}

fn plan_and_price_line(account: &AccountInfo) -> Option<String> {
    let plan = account.plan.as_deref().and_then(plan_name_for_display);
    let price = price_for_display(account);

    match (plan, price) {
        (Some(plan), Some(price)) => Some(format!("{plan} \u{2248} {price}")),
        (Some(plan), None) => Some(plan),
        (None, Some(price)) => Some(format!("\u{2248} {price}")),
        (None, None) => None,
    }
}

fn plan_name_for_display(plan: &str) -> Option<String> {
    let mut characters = plan.trim().chars();
    let first = characters.next()?;

    Some(first.to_uppercase().chain(characters).collect())
}

fn price_for_display(account: &AccountInfo) -> Option<String> {
    let amount = account.price_amount?;
    let price = format_money(amount, account.price_currency.as_deref());

    match account.price_period.as_deref().map(str::trim) {
        Some(period) if !period.is_empty() => Some(format!("{price} /{period}")),
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
    fn plan_and_price_share_the_first_line() {
        let account = AccountInfo {
            plan: Some("pro".to_string()),
            price_amount: Some(20.0),
            price_currency: Some("USD".to_string()),
            price_period: Some("mo".to_string()),
            ..Default::default()
        };

        assert_eq!(
            plan_display_lines(&account, &time_context()),
            vec!["Pro \u{2248} $20.00 /mo".to_string()]
        );
    }

    #[test]
    fn plan_without_price_renders_the_plan_name_alone() {
        let account = AccountInfo {
            plan: Some("plus".to_string()),
            ..Default::default()
        };

        assert_eq!(
            plan_display_lines(&account, &time_context()),
            vec!["Plus".to_string()]
        );
    }

    #[test]
    fn price_without_plan_keeps_the_approximation_sign() {
        let account = AccountInfo {
            price_amount: Some(20.0),
            price_currency: Some("USD".to_string()),
            price_period: Some("mo".to_string()),
            ..Default::default()
        };

        assert_eq!(
            plan_display_lines(&account, &time_context()),
            vec!["\u{2248} $20.00 /mo".to_string()]
        );
    }

    #[test]
    fn neither_plan_nor_price_produces_no_first_line() {
        let account = AccountInfo {
            price_currency: Some("USD".to_string()),
            price_period: Some("mo".to_string()),
            price_note: Some("may vary by country/currency".to_string()),
            ..Default::default()
        };

        assert!(plan_display_lines(&account, &time_context()).is_empty());
    }

    #[test]
    fn price_without_period_renders_as_a_bare_amount() {
        let account = AccountInfo {
            plan: Some("pro".to_string()),
            price_amount: Some(20.0),
            price_currency: Some("USD".to_string()),
            ..Default::default()
        };

        assert_eq!(
            plan_display_lines(&account, &time_context()),
            vec!["Pro \u{2248} $20.00".to_string()]
        );
    }

    #[test]
    fn blank_period_is_treated_as_absent() {
        let account = AccountInfo {
            price_amount: Some(20.0),
            price_currency: Some("USD".to_string()),
            price_period: Some("  ".to_string()),
            ..Default::default()
        };

        assert_eq!(
            plan_display_lines(&account, &time_context()),
            vec!["\u{2248} $20.00".to_string()]
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
    fn non_usd_price_keeps_its_currency_code() {
        let account = AccountInfo {
            price_amount: Some(18.5),
            price_currency: Some("EUR".to_string()),
            price_period: Some("mo".to_string()),
            ..Default::default()
        };

        assert_eq!(
            plan_display_lines(&account, &time_context()),
            vec!["\u{2248} 18.50 EUR /mo".to_string()]
        );
    }

    #[test]
    fn price_note_produces_no_line_of_its_own() {
        let account = AccountInfo {
            plan: Some("pro".to_string()),
            price_amount: Some(20.0),
            price_currency: Some("USD".to_string()),
            price_period: Some("mo".to_string()),
            price_note: Some("may vary by country/currency".to_string()),
            ..Default::default()
        };

        assert_eq!(
            plan_display_lines(&account, &time_context()),
            vec!["Pro \u{2248} $20.00 /mo".to_string()]
        );
    }

    #[test]
    fn renewal_renders_a_lowercase_date_only_line() {
        let context = time_context();
        let account = AccountInfo {
            renewal_at: Some("2026-08-28T12:00:00Z".to_string()),
            ..Default::default()
        };
        let lines = plan_display_lines(&account, &context);

        assert_eq!(
            lines,
            vec![format!(
                "renews {}",
                format_user_date("2026-08-28T12:00:00Z", &context)
            )]
        );
        assert!(lines[0].contains("2026"));
        assert!(!lines[0].contains(':'));
    }

    #[test]
    fn subscription_start_produces_no_line() {
        let account = AccountInfo {
            subscription_started_at: Some("2024-01-12T12:00:00Z".to_string()),
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
    fn full_account_renders_two_lines_in_order() {
        let context = time_context();
        let account = AccountInfo {
            plan: Some("pro".to_string()),
            subscription_started_at: Some("2024-01-12T12:00:00Z".to_string()),
            renewal_at: Some("2026-09-03T12:00:00Z".to_string()),
            price_amount: Some(20.0),
            price_currency: Some("usd".to_string()),
            price_period: Some("mo".to_string()),
            price_note: Some("may vary by country/currency".to_string()),
            plan_management_url: Some("https://example.test/plan".to_string()),
            ..Default::default()
        };

        assert_eq!(
            plan_display_lines(&account, &context),
            vec![
                "Pro \u{2248} $20.00 /mo".to_string(),
                format!(
                    "renews {}",
                    format_user_date("2026-09-03T12:00:00Z", &context)
                ),
            ]
        );
    }
}
