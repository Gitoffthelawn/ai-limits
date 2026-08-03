pub mod app_update;
mod collect;
mod provider_limits;

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use crate::platform::terminal;
use ai_limits::infra::os_access;
use ai_limits::infra::os_access::{
    allowed_cli_command_is_available, CLAUDE_CLI_COMMAND, CODEX_CLI_COMMAND,
};
use ai_limits::types::CliAuthorization;

pub use provider_limits::{ProviderLimits, ProviderLimitsQuery};

use collect::collect_single_provider_limits;

#[tauri::command]
pub fn get_app_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[tauri::command]
pub async fn get_single_provider_limits(
    provider_id: String,
    query: ProviderLimitsQuery,
    app: tauri::AppHandle,
    sent_notifications: tauri::State<'_, Arc<Mutex<HashSet<String>>>>,
) -> Result<ProviderLimits, String> {
    let sent_notifications = Arc::clone(sent_notifications.inner());

    tauri::async_runtime::spawn_blocking(move || {
        collect_single_provider_limits(&provider_id, &query, app, sent_notifications)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn open_external_url(url: String) -> Result<(), String> {
    if !os_access::is_allowed_external_url(&url) {
        return Err("External URL is not allowed".to_string());
    }

    os_access::open_external_url_with_system(&url).map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn start_provider_cli_login(provider: String) -> Result<(), String> {
    let auth = CliAuthorization::parse(&provider)?;
    let cli_command = match auth {
        CliAuthorization::Codex => CODEX_CLI_COMMAND,
        CliAuthorization::Claude => CLAUDE_CLI_COMMAND,
    };

    if !allowed_cli_command_is_available(cli_command) {
        return Err(format!(
            "{cli_command} is not installed or is not available in PATH"
        ));
    }

    terminal::open_with_command(auth.login_command()).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_cli_command() -> Result<String, String> {
    terminal::cli_command_for_current_executable()
}

#[tauri::command]
pub fn run_cli_in_terminal() -> Result<(), String> {
    terminal::run_current_cli_in_terminal()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_authorization_parse_accepts_only_codex_and_claude() {
        assert_eq!(
            CliAuthorization::parse("codex").unwrap().login_command(),
            "codex login"
        );
        assert_eq!(
            CliAuthorization::parse("claude").unwrap().login_command(),
            "claude login"
        );
        assert!(CliAuthorization::parse("cursor").is_err());
        assert!(CliAuthorization::parse("").is_err());
    }
}
