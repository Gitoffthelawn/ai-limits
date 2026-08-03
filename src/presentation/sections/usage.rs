use crate::types::UsageInfo;

use super::super::common::{format_compact_number, format_number};

pub fn usage_display_lines(usage: &UsageInfo) -> Vec<String> {
    let mut lines = Vec::new();

    if let Some(total) = usage.tokens.total {
        lines.push(format!("Tokens {}", format_compact_number(total)));
    }
    for (value, label) in [
        (usage.activity.sessions_count, "Sessions"),
        (usage.activity.turns_count, "Turns"),
        (usage.activity.files_count, "Files"),
    ] {
        if let Some(value) = value {
            lines.push(format!("{label} {}", format_number(value)));
        }
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ActivityUsage, ModelUsage, MoneyUsage, TokenUsage};

    #[test]
    fn empty_usage_produces_no_lines() {
        assert!(usage_display_lines(&UsageInfo::default()).is_empty());
    }

    #[test]
    fn tokens_render_the_total_only() {
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
            vec!["Tokens 711.2M".to_string()]
        );
    }

    #[test]
    fn token_components_without_a_total_produce_no_line() {
        let usage = UsageInfo {
            tokens: TokenUsage {
                input: Some(1_847_433),
                output: Some(3_326_333),
                cache_read: Some(685_869_426),
                ..Default::default()
            },
            ..Default::default()
        };

        assert!(usage_display_lines(&usage).is_empty());
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

        assert_eq!(usage_display_lines(&usage), vec!["Tokens 1.5B".to_string()]);
    }

    #[test]
    fn zero_values_render_as_known_zeros() {
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
            activity: ActivityUsage {
                sessions_count: Some(0),
                ..Default::default()
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
            vec!["Tokens 0".to_string(), "Sessions 0".to_string()]
        );
    }

    #[test]
    fn money_produces_no_lines() {
        let usage = UsageInfo {
            money: MoneyUsage {
                used_amount: Some(12.4),
                remaining_amount: Some(7.6),
                total_amount: Some(20.0),
                currency: Some("USD".to_string()),
            },
            ..Default::default()
        };

        assert!(usage_display_lines(&usage).is_empty());
    }

    #[test]
    fn activity_counts_use_one_line_each_in_a_fixed_order() {
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
            vec![
                "Sessions 92".to_string(),
                "Turns 6,394".to_string(),
                "Files 223".to_string(),
            ]
        );
    }

    #[test]
    fn counts_use_thousands_separators_rather_than_a_compact_suffix() {
        let usage = UsageInfo {
            activity: ActivityUsage {
                turns_count: Some(1_284_311),
                files_count: Some(22_545),
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(
            usage_display_lines(&usage),
            vec!["Turns 1,284,311".to_string(), "Files 22,545".to_string()]
        );
    }

    #[test]
    fn events_latest_activity_and_top_model_produce_no_lines() {
        let usage = UsageInfo {
            activity: ActivityUsage {
                events_count: Some(22_545),
                latest_activity_at: Some("2026-08-02T15:47:37.356Z".to_string()),
                ..Default::default()
            },
            models: ModelUsage {
                top_model: Some("claude-sonnet-5".to_string()),
            },
            ..Default::default()
        };

        assert!(usage_display_lines(&usage).is_empty());
    }

    #[test]
    fn claude_local_shape_renders_the_full_standard_set() {
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
                "Tokens 711.2M".to_string(),
                "Sessions 92".to_string(),
                "Turns 6,394".to_string(),
                "Files 223".to_string(),
            ]
        );
    }

    #[test]
    fn codex_local_shape_renders_tokens_and_files() {
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
            vec!["Tokens 1.5B".to_string(), "Files 921".to_string()]
        );
    }
}
