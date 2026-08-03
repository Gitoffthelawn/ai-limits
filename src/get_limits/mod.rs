mod chain;
mod freshness;
mod plan;

use std::io;

use chrono::Utc;

use crate::providers::{
    claude_cli, claude_local, claude_rpc, codex_cli, codex_local, codex_rpc, cursor_api2,
};
use crate::types::{Source, SourceReport};

pub use plan::{
    best_source_plan, cli_fallback_source_plan, cli_first_source_plan, default_source_plan,
    fast_free_source_plan, source_list_plan, ui_source_plan, SourcePlan, SourcePriority,
    UiSourcePlanOptions,
};

use chain::get_fallback_chain_limits;
use freshness::mark_expired_local_limit_data;

pub fn get_source_plan_limits(plan: SourcePlan) -> io::Result<SourceReport> {
    match plan {
        SourcePlan::Single(source) => get_source_limits(source),
        SourcePlan::Chain { sources, .. } => get_fallback_chain_limits(sources),
    }
}

pub fn get_source_limits(source: Source) -> io::Result<SourceReport> {
    let data = match source {
        Source::CodexLocal => codex_local::get_usage()?,
        Source::CodexRpc => codex_rpc::collect_usage()?,
        Source::CodexCli => codex_cli::collect_usage()?,
        Source::ClaudeRpc => claude_rpc::collect_usage()?,
        Source::ClaudeCli => claude_cli::collect_usage()?,
        Source::ClaudeLocal => claude_local::collect()?,
        Source::CursorApi2 => cursor_api2::collect_usage()?,
    };

    Ok(mark_expired_local_limit_data(
        SourceReport { source, data },
        Utc::now(),
    ))
}
