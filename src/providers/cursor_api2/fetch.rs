use std::io::{self, Write};
use std::process::Stdio;

use crate::infra::os_access::{cursor_usage_request_command, read_cursor_access_token};

pub(super) struct AccessDenied {
    pub message: String,
    pub raw: Option<String>,
}

pub(super) fn fetch_usage_response() -> io::Result<Result<String, AccessDenied>> {
    let token_output = match read_cursor_access_token() {
        Ok(output) => output,
        Err(error) => {
            return Ok(Err(AccessDenied {
                message: format!(
                    "Cursor api2 usage unavailable: cannot read macOS Keychain token ({error})"
                ),
                raw: None,
            }));
        }
    };

    if !token_output.status.success() {
        return Ok(Err(AccessDenied {
            message: "Cursor api2 usage unavailable: token not found; run `cursor agent login`"
                .to_string(),
            raw: None,
        }));
    }

    let token = String::from_utf8_lossy(&token_output.stdout)
        .trim()
        .to_string();
    if token.is_empty() {
        return Ok(Err(AccessDenied {
            message: "Cursor api2 usage unavailable: empty token; run `cursor agent login`"
                .to_string(),
            raw: None,
        }));
    }

    let curl = cursor_usage_request_command()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let mut curl = match curl {
        Ok(child) => child,
        Err(error) => {
            drop(token);
            return Ok(Err(AccessDenied {
                message: format!("Cursor api2 usage unavailable: cannot run curl ({error})"),
                raw: None,
            }));
        }
    };

    if let Some(mut stdin) = curl.stdin.take() {
        stdin.write_all(
            format!(
                "header = \"Authorization: Bearer {token}\"\nheader = \"Content-Type: application/json\"\nheader = \"Connect-Protocol-Version: 1\"\n"
            )
            .as_bytes(),
        )?;
    }

    drop(token);

    let usage_output = match curl.wait_with_output() {
        Ok(output) => output,
        Err(error) => {
            return Ok(Err(AccessDenied {
                message: format!(
                    "Cursor api2 usage unavailable: cannot read curl output ({error})"
                ),
                raw: None,
            }));
        }
    };

    let response = String::from_utf8_lossy(&usage_output.stdout).to_string();

    if !usage_output.status.success() {
        return Ok(Err(AccessDenied {
            message: format!(
                "Cursor api2 usage unavailable: request failed with status {}",
                usage_output.status
            ),
            raw: Some(response),
        }));
    }

    if response.trim().is_empty() {
        return Ok(Err(AccessDenied {
            message: "Cursor api2 usage unavailable: empty response".to_string(),
            raw: Some(response),
        }));
    }

    if response.contains("\"code\":\"unauthenticated\"")
        || response.contains("\"error\":\"unauthorized\"")
        || response.contains("Unauthorized")
    {
        return Ok(Err(AccessDenied {
            message: "Cursor api2 usage unavailable: token rejected; run `cursor agent login`"
                .to_string(),
            raw: Some(response),
        }));
    }

    Ok(Ok(response))
}
