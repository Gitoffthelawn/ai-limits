mod bar;
mod labels;
mod numbers;

pub use bar::{
    pad_visible_left, pad_visible_right, render_limit_bar, ColorConfig, LIMIT_BAR_WIDTH,
    LIMIT_LEFT_WIDTH, LIMIT_WINDOW_WIDTH,
};
pub use labels::{
    format_data_as_of, format_unavailable_block, provider_label, source_label_for_display,
    window_label_for_display, ProviderBlock,
};
pub use numbers::{
    format_compact_number, format_decimal, format_money, format_number, format_percent,
    normalize_percent, remaining_percent_for_display,
};

#[cfg(test)]
pub use bar::{visible_limit_bar, visible_width};
