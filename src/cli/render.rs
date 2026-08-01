use std::io;

use crate::infra::loader::TerminalUi;
use crate::presentation::{
    format_raw_output, format_structured_output, limits_block, usage_block, ColorConfig,
    ProviderBlock,
};
use crate::types::SourceReport;

use super::args::OutputMode;

pub(super) fn print_source_report(
    ui: &mut TerminalUi,
    report: &SourceReport,
    output_mode: OutputMode,
    color: &ColorConfig,
) -> io::Result<()> {
    let block = match output_mode {
        OutputMode::Limits => limits_block(&report.data.structured, color),
        OutputMode::Usage => usage_block(&report.data.structured),
        OutputMode::Raw => ProviderBlock {
            provider_label: report.data.structured.provider.to_ascii_uppercase(),
            body: format_raw_output(&report.data),
        },
        OutputMode::Structured => ProviderBlock {
            provider_label: report.data.structured.provider.to_ascii_uppercase(),
            body: format_structured_output(&report.data),
        },
    };

    ui.print_provider_block(&block.provider_label, &block.body)
}

pub(super) fn failed_source_block(label: &str, error: &str) -> ProviderBlock {
    let provider = match label {
        "codex" | "codex-local" | "codex-cli" => "CODEX",
        "claude" | "claude-cli" | "claude-local" => "CLAUDE",
        "cursor" | "cursor-api2" => "CURSOR",
        _ => "AI LIMITS",
    };

    ProviderBlock {
        provider_label: provider.to_string(),
        body: format!("Unavailable: {error}\nSource {label}: unknown"),
    }
}
