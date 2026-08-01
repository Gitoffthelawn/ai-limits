#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct CursorApiFields {
    pub remaining: Option<f64>,
    pub limit: Option<f64>,
    pub total_percent_used: Option<f64>,
    pub auto_percent_used: Option<f64>,
    pub api_percent_used: Option<f64>,
    pub billing_cycle_start: Option<i64>,
    pub billing_cycle_end: Option<i64>,
    pub display_message: Option<String>,
}

impl CursorApiFields {
    pub(super) fn is_empty(&self) -> bool {
        self.remaining.is_none()
            && self.limit.is_none()
            && self.total_percent_used.is_none()
            && self.auto_percent_used.is_none()
            && self.api_percent_used.is_none()
            && self.billing_cycle_start.is_none()
            && self.billing_cycle_end.is_none()
    }
}

pub(super) fn parse_cursor_api_fields(response: &str) -> CursorApiFields {
    let remaining = json_number_after_key(response, "remaining");
    let limit = json_number_after_key(response, "limit");
    let total_percent_used = json_number_after_key(response, "totalPercentUsed");
    let auto_percent_used = json_number_after_key(response, "autoPercentUsed");
    let api_percent_used = json_number_after_key(response, "apiPercentUsed");
    let billing_cycle_start = json_string_after_key(response, "billingCycleStart")
        .and_then(|value| value.parse::<i64>().ok())
        .or_else(|| json_number_after_key(response, "billingCycleStart").map(|value| value as i64));
    let billing_cycle_end = json_string_after_key(response, "billingCycleEnd")
        .and_then(|value| value.parse::<i64>().ok())
        .or_else(|| json_number_after_key(response, "billingCycleEnd").map(|value| value as i64));
    let display_message = json_string_after_key(response, "displayMessage");

    let fields = CursorApiFields {
        remaining,
        limit,
        total_percent_used,
        auto_percent_used,
        api_percent_used,
        billing_cycle_start,
        billing_cycle_end,
        display_message,
    };

    if fields.is_empty() {
        CursorApiFields::default()
    } else {
        fields
    }
}

fn json_number_after_key(input: &str, key: &str) -> Option<f64> {
    let mut rest = input;
    let needle = format!("\"{key}\"");

    loop {
        let key_index = rest.find(&needle)?;
        let after_key = &rest[key_index + needle.len()..];
        let colon_index = after_key.find(':')?;
        let after_colon = after_key[colon_index + 1..].trim_start();
        let number_len = after_colon
            .chars()
            .take_while(|character| {
                character.is_ascii_digit()
                    || *character == '-'
                    || *character == '+'
                    || *character == '.'
                    || *character == 'e'
                    || *character == 'E'
            })
            .map(char::len_utf8)
            .sum::<usize>();

        if number_len > 0 {
            return after_colon[..number_len].parse::<f64>().ok();
        }

        rest = &after_colon[after_colon.chars().next()?.len_utf8()..];
    }
}

fn json_string_after_key(input: &str, key: &str) -> Option<String> {
    let mut rest = input;
    let needle = format!("\"{key}\"");

    loop {
        let key_index = rest.find(&needle)?;
        let after_key = &rest[key_index + needle.len()..];
        let colon_index = after_key.find(':')?;
        let after_colon = after_key[colon_index + 1..].trim_start();
        if let Some(value) = parse_json_string(after_colon) {
            return Some(value);
        }

        rest = &after_colon[after_colon.chars().next()?.len_utf8()..];
    }
}

fn parse_json_string(input: &str) -> Option<String> {
    let mut chars = input.chars();
    if chars.next()? != '"' {
        return None;
    }

    let mut value = String::new();
    let mut escaped = false;
    for character in chars {
        if escaped {
            value.push(match character {
                '"' => '"',
                '\\' => '\\',
                '/' => '/',
                'b' => '\u{0008}',
                'f' => '\u{000c}',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                other => other,
            });
            escaped = false;
            continue;
        }

        match character {
            '\\' => escaped = true,
            '"' => return Some(value),
            other => value.push(other),
        }
    }

    None
}
