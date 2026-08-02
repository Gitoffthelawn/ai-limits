use std::io;
use std::process::{Command, Stdio};

use crate::infra::os_access::CODEX_CLI_COMMAND;
use crate::infra::process::cli_process_path;

pub(super) fn codex_login_status() -> io::Result<bool> {
    let output = Command::new(CODEX_CLI_COMMAND)
        .args(["login", "status"])
        .env("PATH", cli_process_path())
        .stdin(Stdio::null())
        .output()?;
    Ok(output.status.success()
        && login_status_confirms_authorization(&output.stdout, &output.stderr))
}

pub(super) fn login_status_confirms_authorization(stdout: &[u8], stderr: &[u8]) -> bool {
    [stdout, stderr].into_iter().any(|stream| {
        String::from_utf8_lossy(stream)
            .to_ascii_lowercase()
            .contains("logged in using")
    })
}

pub(super) fn expect_script() -> String {
    format!(
        r#"set timeout 20
log_user 1
spawn env TERM=xterm-256color COLUMNS=120 LINES=40 sh -c {{stty cols 120 rows 40; exec {CODEX_CLI_COMMAND} --no-alt-screen}}
expect {{
    -re {{OpenAI Codex}} {{}}
    timeout {{}}
}}
after 2000
send "\033\[200~/status\033\[201~\r"
expect {{
    -re {{Credits:}} {{set have_usage 1}}
    -re {{refresh requested|5h limit:|Weekly limit:}} {{set have_usage 0}}
    timeout {{set have_usage 0}}
}}
if {{$have_usage == 0}} {{
    after 3000
    send "\033\[200~/status\033\[201~\r"
    expect {{
        -re {{Credits:}} {{}}
        timeout {{}}
    }}
}}
after 1000
set timeout 3
send "\033\[200~/usage\033\[201~\r"
expect {{
    -re {{usage limit reset}} {{}}
    timeout {{}}
}}
after 1000
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
    fn login_status_requires_the_documented_authenticated_output() {
        assert!(login_status_confirms_authorization(
            b"Logged in using ChatGPT\n",
            b""
        ));
        assert!(login_status_confirms_authorization(
            b"",
            b"Logged in using an API key\n"
        ));
        assert!(!login_status_confirms_authorization(
            b"Not logged in\n",
            b""
        ));
        assert!(!login_status_confirms_authorization(
            b"",
            b"Unexpected status output\n"
        ));
    }

    #[test]
    fn usage_script_does_not_start_login() {
        let script = expect_script();

        assert!(!script.contains("codex login"));
        assert!(!script.contains("send \"1\""));
    }

    #[test]
    fn usage_script_never_sends_redemption_input() {
        let script = expect_script();

        assert!(script.contains("/usage"));
        assert!(!script.contains("redeem"));
        assert!(!script.contains("confirm"));
    }
}
