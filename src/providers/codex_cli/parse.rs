use crate::types::LimitInfo;

pub(super) fn output_requires_authorization(raw: &str) -> bool {
    let compact = raw
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase();

    compact.contains("signin")
        || compact.contains("login")
        || compact.contains("notauthenticated")
        || compact.contains("authenticationrequired")
        || compact.contains("authorizationrequired")
}

pub(super) fn normalize_line(raw_line: &str) -> String {
    let line = raw_line
        .trim()
        .trim_matches(|character| character == '\u{2502}')
        .trim();
    strip_progress_bar(line)
}

pub(super) fn parse_limit_line(
    name: &str,
    window_label: &str,
    window_minutes: u64,
    line: &str,
) -> Option<LimitInfo> {
    let remaining_percent = parse_remaining_percent(line)?;
    let used_percent = Some(100.0 - remaining_percent);

    Some(LimitInfo {
        name: name.to_string(),
        window_label: Some(window_label.to_string()),
        window_minutes: Some(window_minutes),
        resets_at: parse_resets_at(line),
        used_percent,
        remaining_percent: Some(remaining_percent),
        used_amount: None,
        remaining_amount: None,
        total_amount: None,
        amount_unit: None,
    })
}

pub(super) fn parse_remaining_percent(line: &str) -> Option<f64> {
    let marker = "% left";
    let percent_end = line.find(marker)?;
    let before_marker = line[..percent_end].trim();
    let value = before_marker.rsplit(' ').next()?;
    value.parse().ok()
}

pub(super) fn parse_resets_at(line: &str) -> Option<String> {
    let marker = "(resets ";
    let start = line.find(marker)? + marker.len();
    let rest = &line[start..];
    let end = rest.find(')')?;
    Some(rest[..end].trim().to_string())
}

pub(super) fn parse_credits_line(line: &str) -> Option<f64> {
    let after_prefix = line.strip_prefix("Credits:")?.trim();
    after_prefix.split_whitespace().next()?.parse().ok()
}

pub(super) fn parse_available_reset_count(line: &str) -> Option<u64> {
    let normalized = line.to_ascii_lowercase();
    if !normalized.contains("usage limit reset") || !normalized.contains("available") {
        return None;
    }

    normalized
        .split_whitespace()
        .find_map(|word| word.parse::<u64>().ok())
}

fn strip_progress_bar(line: &str) -> String {
    let Some(bracket_start) = line.find('[') else {
        return line.to_string();
    };
    let Some(bracket_end) = line[bracket_start..].find(']') else {
        return line.to_string();
    };

    let prefix = line[..bracket_start].trim_end();
    let rest = line[bracket_start + bracket_end + 1..].trim_start();

    if rest.is_empty() {
        prefix.to_string()
    } else {
        format!("{prefix} {rest}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_progress_bar_from_limit_lines() {
        assert_eq!(
            strip_progress_bar("5h limit: [░░░░░░░░░░░░░░░░░░░░] 0% left (resets 07:59)"),
            "5h limit: 0% left (resets 07:59)"
        );
        assert_eq!(
            strip_progress_bar(
                "Weekly limit: [█████████████████░░░] 84% left (resets 02:59 on 6 Jul)"
            ),
            "Weekly limit: 84% left (resets 02:59 on 6 Jul)"
        );
    }

    #[test]
    fn leaves_lines_without_progress_bar_unchanged() {
        assert_eq!(
            strip_progress_bar("5h limit: 0% left (resets 07:59)"),
            "5h limit: 0% left (resets 07:59)"
        );
        assert_eq!(
            strip_progress_bar("Credits: 335 credits"),
            "Credits: 335 credits"
        );
    }
}
