use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use crate::infra::os_access::CODEX_CLI_COMMAND;
use crate::infra::process::cli_process_path;

const APP_SERVER_ARG: &str = "app-server";
const CLIENT_NAME: &str = "ai-limits";
const CLIENT_VERSION: &str = env!("CARGO_PKG_VERSION");

const INITIALIZE_METHOD: &str = "initialize";
const INITIALIZED_NOTIFICATION: &str = "initialized";
const ACCOUNT_METHOD: &str = "account/read";
const RATE_LIMITS_METHOD: &str = "account/rateLimits/read";
const USAGE_METHOD: &str = "account/usage/read";

const INITIALIZE_ID: u64 = 1;
const ACCOUNT_ID: u64 = 2;
const RATE_LIMITS_ID: u64 = 3;
const USAGE_ID: u64 = 4;

const SESSION_TIMEOUT: Duration = Duration::from_secs(30);
const EXIT_TIMEOUT: Duration = Duration::from_secs(2);
const EXIT_POLL_INTERVAL: Duration = Duration::from_millis(50);

/// The raw `result` payloads of the three read-only account methods.
pub(super) struct RpcSession {
    pub(super) account: Value,
    pub(super) rate_limits: Value,
    pub(super) usage: Value,
}

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
            Self::Start => "Codex CLI app-server could not be started",
            Self::Transport => "Codex CLI app-server connection was lost",
            Self::Timeout => "Codex CLI app-server did not respond in time",
            Self::Protocol => "Codex CLI app-server returned an unexpected response",
        }
    }
}

/// Runs the strictly read-only call sequence documented in
/// [codex-rpc-usage.md](../../../docs/get-limits/providers/codex-rpc-usage.md):
/// `initialize`, the `initialized` notification, and the three account reads.
/// Nothing is written to the Codex account and no interactive session starts.
pub(super) fn read_account_session() -> Result<RpcSession, RpcFailure> {
    let mut server = AppServer::start()?;
    let session = server.run_read_only_sequence();
    server.shut_down();
    session
}

/// Owns the child process so that `codex app-server` never outlives collection,
/// including on early returns from the exchange.
struct AppServer {
    child: Child,
    stdin: Option<ChildStdin>,
    lines: Receiver<String>,
}

impl AppServer {
    fn start() -> Result<Self, RpcFailure> {
        let mut child = Command::new(CODEX_CLI_COMMAND)
            .arg(APP_SERVER_ARG)
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

    fn run_read_only_sequence(&mut self) -> Result<RpcSession, RpcFailure> {
        let deadline = Instant::now() + SESSION_TIMEOUT;

        self.request(INITIALIZE_ID, INITIALIZE_METHOD, initialize_params())?;
        self.receive(INITIALIZE_ID, deadline)?;
        self.notify(INITIALIZED_NOTIFICATION)?;

        self.request(ACCOUNT_ID, ACCOUNT_METHOD, json!({}))?;
        let account = self.receive(ACCOUNT_ID, deadline)?;
        self.request(RATE_LIMITS_ID, RATE_LIMITS_METHOD, json!({}))?;
        let rate_limits = self.receive(RATE_LIMITS_ID, deadline)?;
        self.request(USAGE_ID, USAGE_METHOD, json!({}))?;
        let usage = self.receive(USAGE_ID, deadline)?;

        Ok(RpcSession {
            account,
            rate_limits,
            usage,
        })
    }

    fn request(&mut self, id: u64, method: &str, params: Value) -> Result<(), RpcFailure> {
        self.write(json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }))
    }

    fn notify(&mut self, method: &str) -> Result<(), RpcFailure> {
        self.write(json!({ "jsonrpc": "2.0", "method": method, "params": {} }))
    }

    fn write(&mut self, message: Value) -> Result<(), RpcFailure> {
        let stdin = self.stdin.as_mut().ok_or(RpcFailure::Transport)?;
        writeln!(stdin, "{message}").map_err(|_| RpcFailure::Transport)?;
        stdin.flush().map_err(|_| RpcFailure::Transport)
    }

    /// Correlates one response by request id. Notifications, server-initiated
    /// requests, and any non-protocol line are skipped rather than guessed at.
    fn receive(&self, id: u64, deadline: Instant) -> Result<Value, RpcFailure> {
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
            if !is_response_to(&message, id) {
                continue;
            }
            if message.get("error").is_some() {
                return Err(RpcFailure::Protocol);
            }

            return message.get("result").cloned().ok_or(RpcFailure::Protocol);
        }
    }

    /// Closes stdin so the server exits on EOF, then makes sure the child is
    /// gone before returning.
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

impl Drop for AppServer {
    fn drop(&mut self) {
        self.stdin = None;
        if matches!(self.child.try_wait(), Ok(None)) {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }
}

fn initialize_params() -> Value {
    json!({ "clientInfo": { "name": CLIENT_NAME, "version": CLIENT_VERSION } })
}

fn is_response_to(message: &Value, id: u64) -> bool {
    message.get("method").is_none() && message.get("id").and_then(Value::as_u64) == Some(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_params_carry_only_the_client_identity() {
        let params = initialize_params();

        assert_eq!(params["clientInfo"]["name"], CLIENT_NAME);
        assert_eq!(params["clientInfo"]["version"], CLIENT_VERSION);
        assert_eq!(
            params["clientInfo"].as_object().map(|info| info.len()),
            Some(2)
        );
    }

    #[test]
    fn call_sequence_is_limited_to_the_read_only_methods() {
        let methods = [
            INITIALIZE_METHOD,
            INITIALIZED_NOTIFICATION,
            ACCOUNT_METHOD,
            RATE_LIMITS_METHOD,
            USAGE_METHOD,
        ];

        for method in methods {
            assert!(!method.contains("consume"));
            assert!(!method.contains("login"));
            assert!(!method.contains("logout"));
        }
        assert_eq!(methods.len(), 5);
    }

    #[test]
    fn responses_are_correlated_by_request_id() {
        let response = json!({ "id": 3, "result": { "rateLimits": {} } });

        assert!(is_response_to(&response, 3));
        assert!(!is_response_to(&response, 2));
    }

    #[test]
    fn notifications_and_server_requests_are_never_taken_as_responses() {
        let notification = json!({ "method": "remoteControl/status/changed", "params": {} });
        let server_request = json!({ "id": 3, "method": "attestation/generate", "params": {} });

        assert!(!is_response_to(&notification, 3));
        assert!(!is_response_to(&server_request, 3));
    }

    #[test]
    fn failure_messages_are_fixed_literals() {
        for failure in [
            RpcFailure::Start,
            RpcFailure::Transport,
            RpcFailure::Timeout,
            RpcFailure::Protocol,
        ] {
            assert!(failure.message().starts_with("Codex CLI app-server"));
        }
    }
}
