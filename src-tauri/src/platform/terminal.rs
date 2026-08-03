use std::path::Path;
#[cfg(target_os = "macos")]
use std::process::Command;

use ai_limits::infra::os_access;

pub fn cli_command_for_current_executable() -> Result<String, String> {
    #[cfg(not(target_os = "macos"))]
    return Err("CLI command display is currently supported on macOS only".to_string());

    #[cfg(target_os = "macos")]
    cli_command_for_executable(&std::env::current_exe().map_err(|error| error.to_string())?)
}

pub fn run_current_cli_in_terminal() -> Result<(), String> {
    let command = cli_command_for_current_executable()?;
    open_with_command(&command).map_err(|error| error.to_string())
}

pub fn open_with_command(command: &str) -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        let script = format!(
            "tell application \"Terminal\" to do script {}",
            apple_script_string(command)
        );
        Command::new("osascript")
            .args(["-e", &script])
            .spawn()?
            .wait()?;
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = command;
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Opening a terminal is currently supported on macOS only",
        ))
    }
}

fn cli_command_for_executable(executable: &Path) -> Result<String, String> {
    Ok(format!("{} --cli", quoted_executable(executable)?))
}

/// The command is shown in the app and copied by hand, so an app installed
/// under the home directory must not spell out the account name. `~` only
/// expands unquoted, so the home prefix stays outside the quotes.
fn quoted_executable(executable: &Path) -> Result<String, String> {
    let home_relative = os_access::home_relative_path(executable)
        .filter(|rest| !rest.as_os_str().is_empty())
        .map(|rest| utf8_path(&rest).map(|rest| format!("~/{}", shell_quote(rest))));

    match home_relative {
        Some(command) => command,
        None => utf8_path(executable).map(shell_quote),
    }
}

fn utf8_path(path: &Path) -> Result<&str, String> {
    path.to_str()
        .ok_or_else(|| "The app path is not valid UTF-8".to_string())
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

#[cfg(any(target_os = "macos", test))]
fn apple_script_string(value: &str) -> String {
    format!(
        "\"{}\"",
        value
            .replace('\\', "\\\\")
            .replace('\"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
    )
}

#[cfg(test)]
mod tests {
    use super::{apple_script_string, cli_command_for_executable, shell_quote};
    use std::path::Path;

    #[test]
    fn cli_command_quotes_the_running_executable_path() {
        let command = cli_command_for_executable(Path::new(
            "/Applications/AI Limits.app/Contents/MacOS/ai-limits-desktop",
        ));

        assert_eq!(
            command,
            Ok("'/Applications/AI Limits.app/Contents/MacOS/ai-limits-desktop' --cli".to_string())
        );
    }

    #[test]
    fn cli_command_shortens_an_app_installed_under_the_home_directory() {
        let Some(home) = std::env::var_os("HOME").map(std::path::PathBuf::from) else {
            return;
        };
        let command = cli_command_for_executable(
            &home.join("Applications/AI Limits.app/Contents/MacOS/ai-limits-desktop"),
        );

        assert_eq!(
            command,
            Ok("~/'Applications/AI Limits.app/Contents/MacOS/ai-limits-desktop' --cli".to_string())
        );
        assert!(!command.unwrap().contains(&home.display().to_string()));
    }

    #[test]
    fn shell_quote_preserves_apostrophes_in_paths() {
        assert_eq!(
            shell_quote("/Applications/Pat's AI Limits.app"),
            "'/Applications/Pat'\"'\"'s AI Limits.app'"
        );
    }

    #[test]
    fn apple_script_string_escapes_special_characters() {
        assert_eq!(
            apple_script_string("say \\\"hello\\\"\nnext\rline"),
            "\"say \\\\\\\"hello\\\\\\\"\\nnext\\rline\""
        );
    }
}
