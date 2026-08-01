mod capture;
mod parse;
mod project;

use std::io;

use crate::infra::os_access::{allowed_cli_command_is_available, CLAUDE_CLI_COMMAND};
use crate::types::SourceData;

use capture::capture_provider_run;
use project::unavailable_source_data;

pub use project::{build_source_data, structured_from_output};

const PROVIDER: &str = "claude";
const SOURCE: &str = "claude_cli";
const SOURCE_LINK: &str = "https://code.claude.com/docs/en/setup";
const SETUP_LINK: &str = SOURCE_LINK;

pub fn get_usage() -> io::Result<SourceData> {
    if !allowed_cli_command_is_available(CLAUDE_CLI_COMMAND) {
        return Ok(unavailable_source_data(
            None,
            &format!(
                "Claude CLI is not installed or is not available in PATH; install `claude` and try again. Setup: {SETUP_LINK}"
            ),
        ));
    }

    let run = capture_provider_run()?;
    Ok(build_source_data(&run))
}

pub fn collect_usage() -> io::Result<SourceData> {
    get_usage()
}
