use crate::types::{ActivityUsage, MoneyUsage, TokenUsage, UsageInfo};

use super::super::common::{format_compact_number, format_money};

pub fn usage_display_lines(usage: &UsageInfo) -> Vec<String> {
    let mut lines = Vec::new();

    if let Some(line) = tokens_line(&usage.tokens) {
        lines.push(line);
    }
    if let Some(line) = money_line(&usage.money) {
        lines.push(line);
    }
    if let Some(line) = activity_line(&usage.activity) {
        lines.push(line);
    }
    if let Some(model) = usage.models.top_model.as_deref() {
        lines.push(format!("Top model: {model}"));
    }

    lines
}

fn tokens_line(tokens: &TokenUsage) -> Option<String> {
    if let Some(total) = tokens.total {
        return Some(format!("Tokens: {} total", format_compact_number(total)));
    }

    let parts = [
        (tokens.input, "input"),
        (tokens.output, "output"),
        (tokens.cached_input, "cached input"),
        (tokens.cache_read, "cache read"),
        (tokens.cache_write, "cache write"),
        (tokens.reasoning_output, "reasoning"),
    ]
    .into_iter()
    .filter_map(|(value, label)| {
        value.map(|value| format!("{} {label}", format_compact_number(value)))
    })
    .collect::<Vec<_>>();

    join_parts(parts).map(|joined| format!("Tokens: {joined}"))
}

fn money_line(money: &MoneyUsage) -> Option<String> {
    let currency = money.currency.as_deref();

    if let Some(amount) = money.used_amount.or(money.total_amount) {
        return Some(format!(
            "Spend this period: {}",
            format_money(amount, currency)
        ));
    }

    money
        .remaining_amount
        .map(|amount| format!("Remaining: {}", format_money(amount, currency)))
}

fn activity_line(activity: &ActivityUsage) -> Option<String> {
    let parts = [
        (activity.sessions_count, "Sessions"),
        (activity.turns_count, "Turns"),
        (activity.files_count, "Files"),
        (activity.events_count, "Events"),
    ]
    .into_iter()
    .filter_map(|(value, label)| {
        value.map(|value| format!("{label}: {}", format_compact_number(value)))
    })
    .collect::<Vec<_>>();

    join_parts(parts)
}

fn join_parts(parts: Vec<String>) -> Option<String> {
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" \u{b7} "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ModelUsage;

    #[test]
    fn empty_usage_produces_no_lines() {
        assert!(usage_display_lines(&UsageInfo::default()).is_empty());
    }

    #[test]
    fn tokens_lead_with_total_when_available() {
        let usage = UsageInfo {
            tokens: TokenUsage {
                input: Some(1_847_433),
                output: Some(3_326_333),
                cache_read: Some(685_869_426),
                cache_write: Some(20_146_310),
                total: Some(711_189_502),
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(
            usage_display_lines(&usage),
            vec!["Tokens: 711.2M total".to_string()]
        );
    }

    #[test]
    fn tokens_compose_from_components_without_total() {
        let usage = UsageInfo {
            tokens: TokenUsage {
                input: Some(1_847_433),
                output: Some(3_326_333),
                cache_read: Some(685_869_426),
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(
            usage_display_lines(&usage),
            vec!["Tokens: 1.8M input \u{b7} 3.3M output \u{b7} 685.9M cache read".to_string()]
        );
    }

    #[test]
    fn billion_scale_totals_use_a_compact_suffix() {
        let usage = UsageInfo {
            tokens: TokenUsage {
                total: Some(1_545_504_962),
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(
            usage_display_lines(&usage),
            vec!["Tokens: 1.5B total".to_string()]
        );
    }

    #[test]
    fn zero_tokens_render_as_a_known_zero() {
        let usage = UsageInfo {
            tokens: TokenUsage {
                input: Some(0),
                cached_input: Some(0),
                output: Some(0),
                reasoning_output: Some(0),
                cache_read: Some(0),
                cache_write: Some(0),
                total: Some(0),
            },
            money: MoneyUsage {
                used_amount: Some(0.0),
                total_amount: Some(0.0),
                currency: Some("usd".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(
            usage_display_lines(&usage),
            vec![
                "Tokens: 0 total".to_string(),
                "Spend this period: $0.00".to_string(),
            ]
        );
    }

    #[test]
    fn money_falls_back_to_total_amount() {
        let usage = UsageInfo {
            money: MoneyUsage {
                total_amount: Some(20.0),
                currency: Some("USD".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(
            usage_display_lines(&usage),
            vec!["Spend this period: $20.00".to_string()]
        );
    }

    #[test]
    fn money_prefers_used_amount_over_total_amount() {
        let usage = UsageInfo {
            money: MoneyUsage {
                used_amount: Some(12.4),
                total_amount: Some(20.0),
                currency: Some("usd".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(
            usage_display_lines(&usage),
            vec!["Spend this period: $12.40".to_string()]
        );
    }

    #[test]
    fn money_without_spend_falls_back_to_remaining_amount() {
        let usage = UsageInfo {
            money: MoneyUsage {
                remaining_amount: Some(7.5),
                currency: Some("EUR".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(
            usage_display_lines(&usage),
            vec!["Remaining: 7.50 EUR".to_string()]
        );
    }

    #[test]
    fn currency_alone_produces_no_money_line() {
        let usage = UsageInfo {
            money: MoneyUsage {
                currency: Some("usd".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        assert!(usage_display_lines(&usage).is_empty());
    }

    #[test]
    fn activity_combines_counters_on_one_line() {
        let usage = UsageInfo {
            activity: ActivityUsage {
                files_count: Some(223),
                sessions_count: Some(92),
                turns_count: Some(6_394),
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(
            usage_display_lines(&usage),
            vec!["Sessions: 92 \u{b7} Turns: 6,394 \u{b7} Files: 223".to_string()]
        );
    }

    #[test]
    fn activity_uses_thousands_separators() {
        let usage = UsageInfo {
            activity: ActivityUsage {
                events_count: Some(22_545),
                files_count: Some(921),
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(
            usage_display_lines(&usage),
            vec!["Files: 921 \u{b7} Events: 22,545".to_string()]
        );
    }

    #[test]
    fn latest_activity_alone_produces_no_line() {
        let usage = UsageInfo {
            activity: ActivityUsage {
                latest_activity_at: Some("2026-08-02T15:47:37.356Z".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        assert!(usage_display_lines(&usage).is_empty());
    }

    #[test]
    fn models_render_the_top_model() {
        let usage = UsageInfo {
            models: ModelUsage {
                top_model: Some("claude-sonnet-5".to_string()),
            },
            ..Default::default()
        };

        assert_eq!(
            usage_display_lines(&usage),
            vec!["Top model: claude-sonnet-5".to_string()]
        );
    }

    #[test]
    fn claude_local_shape_renders_one_line_per_group() {
        let usage = UsageInfo {
            tokens: TokenUsage {
                input: Some(1_847_433),
                output: Some(3_326_333),
                cache_read: Some(685_869_426),
                cache_write: Some(20_146_310),
                total: Some(711_189_502),
                ..Default::default()
            },
            activity: ActivityUsage {
                files_count: Some(223),
                sessions_count: Some(92),
                turns_count: Some(6_394),
                ..Default::default()
            },
            models: ModelUsage {
                top_model: Some("claude-sonnet-5".to_string()),
            },
            ..Default::default()
        };

        assert_eq!(
            usage_display_lines(&usage),
            vec![
                "Tokens: 711.2M total".to_string(),
                "Sessions: 92 \u{b7} Turns: 6,394 \u{b7} Files: 223".to_string(),
                "Top model: claude-sonnet-5".to_string(),
            ]
        );
    }

    #[test]
    fn codex_local_shape_renders_tokens_and_activity() {
        let usage = UsageInfo {
            tokens: TokenUsage {
                input: Some(1_538_117_126),
                cached_input: Some(1_440_486_272),
                output: Some(7_290_916),
                reasoning_output: Some(1_386_692),
                total: Some(1_545_504_962),
                ..Default::default()
            },
            activity: ActivityUsage {
                events_count: Some(22_545),
                files_count: Some(921),
                latest_activity_at: Some("2026-08-02T15:47:37.356Z".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(
            usage_display_lines(&usage),
            vec![
                "Tokens: 1.5B total".to_string(),
                "Files: 921 \u{b7} Events: 22,545".to_string(),
            ]
        );
    }
}
