use std::io;

use crate::infra::os_access::CLAUDE_CLI_COMMAND;
use crate::infra::process::run_provider;
use crate::types::ProviderRun;

pub(super) fn capture_provider_run() -> io::Result<ProviderRun> {
    run_provider(&expect_script())
}

fn expect_script() -> String {
    format!(
        r#"set timeout 25
log_user 1
spawn env TERM=xterm-256color COLUMNS=120 LINES=40 sh -c {{stty cols 120 rows 40; exec {CLAUDE_CLI_COMMAND} --no-chrome}}
expect {{
    -re {{Choose.*text.*style|Syntax theme}} {{
        send "\r"
        exp_continue
    }}
    -re {{for shortcuts|Do you trust|Select login method}} {{}}
    timeout {{}}
}}
after 500
send -- "/usage\r"
expect {{
    -re {{Usage|Current session|Current week|Resets}} {{}}
    -re {{Select login method|Choose.*text.*style}} {{}}
    timeout {{}}
}}
after 10000
send "\003"
after 500
send "\003"
expect {{
    eof {{}}
    timeout {{exit 0}}
}}
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_script_does_not_start_login() {
        let script = expect_script();

        assert!(script.contains("--no-chrome"));
        assert!(!script.contains("claude login"));
        assert!(!script.contains("send -- \"1\""));
    }
}
