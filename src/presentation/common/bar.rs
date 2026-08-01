pub struct ColorConfig {
    pub enabled: bool,
}

impl ColorConfig {
    pub fn from_env(is_tty: bool) -> Self {
        let disabled = std::env::var_os("NO_COLOR").is_some();
        Self {
            enabled: is_tty && !disabled,
        }
    }
}

fn color_for_remaining(remaining_percent: f64, color: &ColorConfig) -> &'static str {
    if !color.enabled {
        return "";
    }

    if remaining_percent >= 75.0 {
        "\x1b[32m"
    } else if remaining_percent >= 50.0 {
        "\x1b[33m"
    } else if remaining_percent >= 25.0 {
        "\x1b[38;5;208m"
    } else if remaining_percent >= 10.0 {
        "\x1b[31m"
    } else {
        "\x1b[91m"
    }
}

const COLOR_RESET: &str = "\x1b[0m";

pub const LIMIT_WINDOW_WIDTH: usize = 4;
pub const LIMIT_BAR_WIDTH: usize = 25;
pub const LIMIT_LEFT_WIDTH: usize = 11;

pub fn visible_width(text: &str) -> usize {
    strip_ansi(text).chars().count()
}

pub fn pad_visible_right(text: &str, width: usize) -> String {
    let padding = width.saturating_sub(visible_width(text));
    format!("{text}{}", " ".repeat(padding))
}

pub fn pad_visible_left(text: &str, width: usize) -> String {
    let padding = width.saturating_sub(visible_width(text));
    format!("{}{text}", " ".repeat(padding))
}

fn strip_ansi(text: &str) -> String {
    let mut stripped = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(character) = chars.next() {
        if character == '\x1b' {
            for next in chars.by_ref() {
                if next == 'm' {
                    break;
                }
            }
            continue;
        }
        stripped.push(character);
    }

    stripped
}

pub fn render_limit_bar(remaining_percent: f64, color: &ColorConfig) -> String {
    let visible = visible_limit_bar(remaining_percent);
    colorize_limit_bar(&visible, remaining_percent, color)
}

pub fn visible_limit_bar(remaining_percent: f64) -> String {
    let clamped = remaining_percent.clamp(0.0, 100.0);
    let full_blocks = ((clamped / 100.0) * 25.0).round() as usize;
    let full_blocks = full_blocks.min(25);
    let mut bar = String::with_capacity(25);

    for _ in 0..full_blocks {
        bar.push('■');
    }
    for _ in full_blocks..25 {
        bar.push('□');
    }

    bar
}

fn colorize_limit_bar(visible: &str, remaining_percent: f64, color: &ColorConfig) -> String {
    if !color.enabled {
        return visible.to_string();
    }

    let color_code = color_for_remaining(remaining_percent, color);
    if color_code.is_empty() {
        return visible.to_string();
    }

    let mut colored = String::new();
    let mut in_fill = false;
    for character in visible.chars() {
        let is_filled = character == '■';
        if is_filled && !in_fill {
            colored.push_str(color_code);
            in_fill = true;
        } else if !is_filled && in_fill {
            colored.push_str(COLOR_RESET);
            in_fill = false;
        }
        colored.push(character);
    }
    if in_fill {
        colored.push_str(COLOR_RESET);
    }

    colored
}
