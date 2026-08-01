use crate::types::{LimitInfo, MoneyUsage, TokenUsage, UsageInfo};

pub(super) struct ParsedClaudeCliOutput {
    pub(super) limits: Vec<LimitInfo>,
    pub(super) usage: UsageInfo,
    pub(super) setup_required: bool,
    pub(super) diagnostics: Vec<String>,
}

impl ParsedClaudeCliOutput {
    pub(super) fn has_usage_data(&self) -> bool {
        !self.limits.is_empty()
            || self.usage.money.used_amount.is_some()
            || self.usage.tokens.input.is_some()
            || self.usage.tokens.output.is_some()
            || self.usage.tokens.cache_read.is_some()
            || self.usage.tokens.cache_write.is_some()
    }
}

pub(super) fn parse_claude_cli_output(input: &str) -> ParsedClaudeCliOutput {
    let lines = normalize_lines(input);

    if lines.iter().any(|line| {
        let compact = compact_for_matching(line);
        compact.contains("selectloginmethod") || compact.contains("choosethetextstyle")
    }) {
        return ParsedClaudeCliOutput {
            limits: Vec::new(),
            usage: UsageInfo::default(),
            setup_required: true,
            diagnostics: Vec::new(),
        };
    }

    let mut limits = Vec::new();
    if let Some(limit) = structured_limit_block(&lines, "Current session", Some(300)) {
        limits.push(limit);
    }
    if let Some(limit) = structured_limit_block(&lines, "Current week", Some(10080)) {
        limits.push(limit);
    }

    let mut usage = UsageInfo::default();
    if let Some(line) = find_line_by_compact_prefix(&lines, "totalcost") {
        usage.money = parse_money_line(&line);
    }
    if let Some(line) = find_line_by_compact_prefix(&lines, "usage") {
        usage.tokens = parse_token_usage_line(&line);
    }

    ParsedClaudeCliOutput {
        limits,
        usage,
        setup_required: false,
        diagnostics: Vec::new(),
    }
}

fn structured_limit_block(
    lines: &[String],
    label: &str,
    window_minutes: Option<u64>,
) -> Option<LimitInfo> {
    let label_compact = compact_for_matching(label);
    let start = lines
        .iter()
        .position(|line| compact_for_matching(line).starts_with(&label_compact))?;

    let used_percent = lines
        .iter()
        .skip(start + 1)
        .take(3)
        .find_map(|line| extract_percent_used(line))
        .and_then(|value| parse_percent_f64(&value));

    let resets_at = lines
        .iter()
        .skip(start + 1)
        .take(5)
        .find_map(|line| extract_resets(line));

    let (used_percent, remaining_percent) = complement_percents(used_percent, None);

    Some(LimitInfo {
        name: label.to_string(),
        window_label: Some(label.to_string()),
        window_minutes,
        resets_at,
        used_percent,
        remaining_percent,
        used_amount: None,
        remaining_amount: None,
        total_amount: None,
        amount_unit: None,
    })
}

fn complement_percents(
    used_percent: Option<f64>,
    remaining_percent: Option<f64>,
) -> (Option<f64>, Option<f64>) {
    match (used_percent, remaining_percent) {
        (Some(used), None) => (Some(used), Some((100.0 - used).max(0.0))),
        (None, Some(remaining)) => (Some((100.0 - remaining).max(0.0)), Some(remaining)),
        (used, remaining) => (used, remaining),
    }
}

fn parse_percent_f64(value: &str) -> Option<f64> {
    value.trim_end_matches('%').parse::<f64>().ok()
}

fn parse_money_line(line: &str) -> MoneyUsage {
    let Some((_, value)) = line.split_once(':') else {
        return MoneyUsage::default();
    };

    let value = value.trim();
    let numeric = value.trim_start_matches('$').trim().parse::<f64>().ok();

    MoneyUsage {
        used_amount: numeric,
        remaining_amount: None,
        total_amount: numeric,
        currency: if value.starts_with('$') {
            Some("usd".to_string())
        } else {
            None
        },
    }
}

fn parse_token_usage_line(line: &str) -> TokenUsage {
    let Some((_, value)) = line.split_once(':') else {
        return TokenUsage::default();
    };

    let mut tokens = TokenUsage::default();

    for segment in value.split(',') {
        let compact = compact_for_matching(segment);
        let amount = extract_leading_number(&compact);

        if compact.contains("input") && !compact.contains("cache") {
            tokens.input = amount;
        } else if compact.contains("output") {
            tokens.output = amount;
        } else if compact.contains("cacheread") {
            tokens.cache_read = amount;
        } else if compact.contains("cachewrite") {
            tokens.cache_write = amount;
        }
    }

    if tokens.input.is_some()
        || tokens.output.is_some()
        || tokens.cache_read.is_some()
        || tokens.cache_write.is_some()
    {
        tokens.total = Some(
            tokens.input.unwrap_or(0)
                + tokens.output.unwrap_or(0)
                + tokens.cache_read.unwrap_or(0)
                + tokens.cache_write.unwrap_or(0),
        );
    }

    tokens
}

fn extract_leading_number(compact_segment: &str) -> Option<u64> {
    let digits = compact_segment
        .chars()
        .take_while(|character| character.is_ascii_digit())
        .collect::<String>();

    if digits.is_empty() {
        None
    } else {
        digits.parse().ok()
    }
}

fn normalize_lines(input: &str) -> Vec<String> {
    input
        .split(['\n', '\r'])
        .map(normalize_terminal_line)
        .filter(|line| !line.is_empty())
        .collect()
}

fn normalize_terminal_line(raw_line: &str) -> String {
    raw_line
        .trim()
        .trim_matches(|character| character == '\u{2502}')
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn extract_percent_used(line: &str) -> Option<String> {
    let used_index = line.find("used")?;
    let before_used = &line[..used_index];
    let percent_index = before_used.rfind('%')?;
    let digits = before_used[..percent_index]
        .chars()
        .rev()
        .take_while(|character| character.is_ascii_digit())
        .collect::<String>();

    if digits.is_empty() {
        return None;
    }

    Some(digits.chars().rev().collect::<String>() + "%")
}

fn extract_resets(line: &str) -> Option<String> {
    let compact = compact_for_matching(line);
    if !compact.starts_with("resets") {
        return None;
    }

    line.split_once(' ')
        .map(|(_, value)| value.trim().to_string())
        .or_else(|| {
            line.strip_prefix("Resets")
                .map(|value| value.trim().to_string())
        })
        .filter(|value| !value.is_empty())
}

fn find_line_by_compact_prefix(lines: &[String], prefix: &str) -> Option<String> {
    lines
        .iter()
        .find(|line| compact_for_matching(line).starts_with(prefix))
        .cloned()
}

fn compact_for_matching(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase()
}
