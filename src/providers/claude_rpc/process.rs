use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use crate::infra::os_access::CLAUDE_CLI_COMMAND;
use crate::infra::process::cli_process_path;

/// Print mode with the stream-json control protocol on both pipes.
///
/// `--verbose` is mandatory, not cosmetic: without it the CLI refuses
/// `--output-format stream-json` and exits without ever answering.
/// `--no-session-persistence` keeps the run from writing a transcript, so the
/// collection does not pollute what `claude_local` reads.
const PRINT_MODE_ARGS: [&str; 6] = [
    "-p",
    "--verbose",
    "--input-format",
    "stream-json",
    "--output-format",
    "stream-json",
];
const NO_SESSION_PERSISTENCE_ARG: &str = "--no-session-persistence";

const CONTROL_REQUEST_TYPE: &str = "control_request";
const CONTROL_RESPONSE_TYPE: &str = "control_response";
const GET_USAGE_SUBTYPE: &str = "get_usage";
const SUCCESS_SUBTYPE: &str = "success";
const REQUEST_ID: &str = "1";

const SESSION_TIMEOUT: Duration = Duration::from_secs(30);
const EXIT_TIMEOUT: Duration = Duration::from_secs(2);
const EXIT_POLL_INTERVAL: Duration = Duration::from_millis(50);

/// Transport failures never carry source content: each variant maps to one
/// fixed literal, so no response body, error payload, or stderr text can reach
/// a user-visible string.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RpcFailure {
    Start,
    Transport,
    Timeout,
    Protocol,
}

impl RpcFailure {
    pub(super) fn message(self) -> &'static str {
        match self {
            Self::Start => "Claude CLI usage request could not be started",
            Self::Transport => "Claude CLI usage request lost its connection",
            Self::Timeout => "Claude CLI usage request did not respond in time",
            Self::Protocol => "Claude CLI usage request returned an unexpected response",
        }
    }
}

/// Runs the strictly read-only call sequence documented in
/// [claude-rpc-usage.md](../../../docs/get-limits/providers/claude-rpc-usage.md):
/// one `get_usage` control request, one correlated control response, then
/// stdin is closed and the process is allowed to exit. No prompt is submitted,
/// no slash command is sent, and nothing is written to the Claude account.
pub(super) fn read_usage() -> Result<Value, RpcFailure> {
    let mut session = PrintModeSession::start()?;
    let usage = session.request_usage();
    session.shut_down();
    usage
}

/// Owns the child process so that `claude` never outlives collection,
/// including on early returns from the exchange.
struct PrintModeSession {
    child: Child,
    stdin: Option<ChildStdin>,
    lines: Receiver<String>,
}

impl PrintModeSession {
    fn start() -> Result<Self, RpcFailure> {
        let mut child = Command::new(CLAUDE_CLI_COMMAND)
            .args(PRINT_MODE_ARGS)
            .arg(NO_SESSION_PERSISTENCE_ARG)
            .env("PATH", cli_process_path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| RpcFailure::Start)?;

        let stdin = child.stdin.take().ok_or(RpcFailure::Start)?;
        let stdout = child.stdout.take().ok_or(RpcFailure::Start)?;
        let (sender, lines) = mpsc::channel();
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if sender.send(line).is_err() {
                    break;
                }
            }
        });

        Ok(Self {
            child,
            stdin: Some(stdin),
            lines,
        })
    }

    /// Writes the request and reads the answer while stdin is still open. The
    /// CLI answers either way, but keeping stdin open until the response has
    /// been read means an EOF can never race the answer.
    fn request_usage(&mut self) -> Result<Value, RpcFailure> {
        let deadline = Instant::now() + SESSION_TIMEOUT;
        self.write(usage_request())?;
        self.receive(deadline)
    }

    fn write(&mut self, message: Value) -> Result<(), RpcFailure> {
        let stdin = self.stdin.as_mut().ok_or(RpcFailure::Transport)?;
        writeln!(stdin, "{message}").map_err(|_| RpcFailure::Transport)?;
        stdin.flush().map_err(|_| RpcFailure::Transport)
    }

    /// Correlates the control response by `request_id`. Assistant messages,
    /// system events, and any non-protocol line are skipped rather than
    /// guessed at.
    fn receive(&self, deadline: Instant) -> Result<Value, RpcFailure> {
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(RpcFailure::Timeout);
            }

            let line = match self.lines.recv_timeout(remaining) {
                Ok(line) => line,
                Err(RecvTimeoutError::Timeout) => return Err(RpcFailure::Timeout),
                Err(RecvTimeoutError::Disconnected) => return Err(RpcFailure::Transport),
            };

            let Ok(message) = serde_json::from_str::<Value>(&line) else {
                continue;
            };
            if !is_usage_response(&message) {
                continue;
            }

            return usage_payload(&message).ok_or(RpcFailure::Protocol);
        }
    }

    /// Closes stdin so the CLI exits on EOF, then makes sure the child is gone
    /// before returning.
    fn shut_down(&mut self) {
        self.stdin = None;

        let deadline = Instant::now() + EXIT_TIMEOUT;
        while Instant::now() < deadline {
            if matches!(self.child.try_wait(), Ok(Some(_)) | Err(_)) {
                return;
            }
            thread::sleep(EXIT_POLL_INTERVAL);
        }

        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for PrintModeSession {
    fn drop(&mut self) {
        self.stdin = None;
        if matches!(self.child.try_wait(), Ok(None)) {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }
}

/// The only line this source ever writes. No handshake precedes it: the
/// control handler answers `get_usage` without an `initialize` request.
fn usage_request() -> Value {
    json!({
        "type": CONTROL_REQUEST_TYPE,
        "request_id": REQUEST_ID,
        "request": { "subtype": GET_USAGE_SUBTYPE }
    })
}

fn is_usage_response(message: &Value) -> bool {
    message.get("type").and_then(Value::as_str) == Some(CONTROL_RESPONSE_TYPE)
        && message
            .pointer("/response/request_id")
            .and_then(Value::as_str)
            == Some(REQUEST_ID)
}

/// The payload is taken only from a response the CLI itself marks successful;
/// an error response degrades to a protocol failure rather than to a partial
/// read of an error body.
fn usage_payload(message: &Value) -> Option<Value> {
    let response = message.get("response")?;
    if response.get("subtype").and_then(Value::as_str) != Some(SUCCESS_SUBTYPE) {
        return None;
    }
    response.get("response").cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_transport_uses_print_mode_with_mandatory_verbose_and_no_persistence() {
        assert!(PRINT_MODE_ARGS.contains(&"--verbose"));
        assert!(PRINT_MODE_ARGS.contains(&"-p"));
        assert_eq!(NO_SESSION_PERSISTENCE_ARG, "--no-session-persistence");
        assert_eq!(
            PRINT_MODE_ARGS
                .iter()
                .filter(|arg| **arg == "stream-json")
                .count(),
            2
        );
    }

    #[test]
    fn the_only_written_line_is_the_read_only_usage_request() {
        let request = usage_request();

        assert_eq!(request["type"], CONTROL_REQUEST_TYPE);
        assert_eq!(request["request"]["subtype"], GET_USAGE_SUBTYPE);
        assert_eq!(
            request["request"].as_object().map(|value| value.len()),
            Some(1)
        );
        let serialized = request.to_string();
        for forbidden in ["prompt", "usage-credits", "login", "logout", "/"] {
            assert!(!serialized.contains(forbidden));
        }
    }

    #[test]
    fn responses_are_correlated_by_request_id() {
        let matching = json!({
            "type": "control_response",
            "response": { "subtype": "success", "request_id": "1", "response": {} }
        });
        let other = json!({
            "type": "control_response",
            "response": { "subtype": "success", "request_id": "2", "response": {} }
        });

        assert!(is_usage_response(&matching));
        assert!(!is_usage_response(&other));
    }

    #[test]
    fn stream_messages_are_never_taken_as_the_control_response() {
        let assistant = json!({ "type": "assistant", "message": { "content": [] } });
        let system = json!({ "type": "system", "subtype": "init" });

        assert!(!is_usage_response(&assistant));
        assert!(!is_usage_response(&system));
    }

    #[test]
    fn only_a_success_response_yields_a_payload() {
        let success = json!({
            "type": "control_response",
            "response": { "subtype": "success", "request_id": "1", "response": { "subscription_type": "pro" } }
        });
        let error = json!({
            "type": "control_response",
            "response": { "subtype": "error", "request_id": "1", "error": "boom" }
        });

        assert_eq!(
            usage_payload(&success),
            Some(json!({ "subscription_type": "pro" }))
        );
        assert_eq!(usage_payload(&error), None);
    }

    #[test]
    fn failure_messages_are_fixed_literals() {
        for failure in [
            RpcFailure::Start,
            RpcFailure::Transport,
            RpcFailure::Timeout,
            RpcFailure::Protocol,
        ] {
            assert!(failure.message().starts_with("Claude CLI usage request"));
        }
    }
}
