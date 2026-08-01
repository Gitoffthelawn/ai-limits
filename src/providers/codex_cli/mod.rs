mod parse;
mod process;
mod project;

use std::io;

use crate::infra::os_access::{allowed_cli_command_is_available, CODEX_CLI_COMMAND};
use crate::infra::process::run_provider;
use crate::types::SourceData;

pub use project::build_structured;

use process::{codex_login_status, expect_script};
use project::{authorization_required_source_data, unavailable_source_data, SETUP_LINK};

pub fn collect_usage() -> io::Result<SourceData> {
    if !allowed_cli_command_is_available(CODEX_CLI_COMMAND) {
        return Ok(unavailable_source_data(
            None,
            &format!(
                "Codex CLI is not installed or is not available in PATH; install `codex` and try again. Setup: {SETUP_LINK}"
            ),
            None,
        ));
    }

    match codex_login_status() {
        Ok(true) => {}
        Ok(false) => return Ok(authorization_required_source_data()),
        Err(error) => {
            return Ok(unavailable_source_data(
                None,
                &format!("Could not check Codex CLI sign-in status: {error}"),
                None,
            ));
        }
    }

    let run = run_provider(&expect_script())?;
    let raw = run.compacted_stdout;
    let mut structured = build_structured(&raw);

    if !run.stderr.trim().is_empty() {
        structured
            .diagnostics
            .push(format!("stderr: {}", run.stderr.trim()));
    }

    Ok(SourceData {
        raw: Some(raw),
        structured,
        stderr: run.stderr,
    })
}
