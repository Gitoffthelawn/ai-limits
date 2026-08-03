use std::env;
use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub const CURSOR_ACCESS_TOKEN_SERVICE: &str = "cursor-access-token";
pub const CURSOR_DASHBOARD_URL_PREFIX: &str =
    "https://api2.cursor.sh/aiserver.v1.DashboardService/";

/// Every Cursor dashboard method this app may call. All of them are `Get*`
/// methods and are read-only; nothing that changes the account is reachable.
pub const CURSOR_DASHBOARD_METHODS: &[&str] = &[
    "GetPlanInfo",
    "GetCurrentPeriodUsage",
    "GetHardLimit",
    "GetAggregatedUsageEvents",
    "GetFilteredUsageEvents",
    "GetMe",
    "GetMonthlyBillingCycle",
];

pub const CLAUDE_CLI_COMMAND: &str = "claude";
pub const CODEX_CLI_COMMAND: &str = "codex";
pub const EXPECT_COMMAND: &str = "expect";

pub const ALLOWED_EXTERNAL_URLS: &[&str] = &[
    "https://code.claude.com/docs/en/setup",
    "https://developers.openai.com/codex/cli",
    "https://github.com/md2it/ai-limits",
    "https://github.com/md2it/ai-limits/blob/main/LICENSE",
];

pub fn codex_local_root() -> io::Result<PathBuf> {
    if let Some(value) = env::var_os("CODEX_HOME") {
        return Ok(PathBuf::from(value));
    }

    let home = env::var_os("HOME").ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "HOME is not set; cannot locate ${CODEX_HOME:-~/.codex}",
        )
    })?;

    Ok(PathBuf::from(home).join(".codex"))
}

fn home_dir(context: &'static str) -> io::Result<PathBuf> {
    let home =
        env::var_os("HOME").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, context))?;

    Ok(PathBuf::from(home))
}

/// The part of `path` below the home directory, or `None` when the path is not
/// inside it. The comparison runs over path components, so a sibling that only
/// shares a string prefix (`/Users/pat-backup` next to `/Users/pat`) is not a
/// match. An unset or empty `HOME` means nothing can be attributed to the home
/// directory, so the answer is `None`.
pub fn home_relative_path(path: &Path) -> Option<PathBuf> {
    relative_to_home(path, home_dir("HOME is not set").ok().as_deref())
}

/// Renders a path for output. An absolute path inside the home directory
/// carries the account name, so the home prefix collapses to `~`; every other
/// path is rendered unchanged. This is a presentation helper only — reads and
/// scans keep using the original path.
pub fn display_path(path: &Path) -> String {
    render_path(path, home_dir("HOME is not set").ok().as_deref())
}

fn relative_to_home(path: &Path, home: Option<&Path>) -> Option<PathBuf> {
    let home = home?;
    home.components().next()?;

    path.strip_prefix(home).ok().map(Path::to_path_buf)
}

fn render_path(path: &Path, home: Option<&Path>) -> String {
    match relative_to_home(path, home) {
        Some(rest) if rest.as_os_str().is_empty() => "~".to_string(),
        Some(rest) => Path::new("~").join(rest).display().to_string(),
        None => path.display().to_string(),
    }
}

/// `~/.claude.json` holds the Claude profile cache and the cached `/usage`
/// snapshot. It sits in the home directory itself, not inside `~/.claude`.
pub fn claude_local_profile_path() -> io::Result<PathBuf> {
    Ok(home_dir("HOME is not set; cannot locate ~/.claude.json")?.join(".claude.json"))
}

pub fn claude_local_stats_cache_path() -> io::Result<PathBuf> {
    let home = home_dir("HOME is not set; cannot locate ~/.claude/stats-cache.json")?;

    Ok(home.join(".claude").join("stats-cache.json"))
}

pub fn claude_local_roots() -> io::Result<Vec<PathBuf>> {
    let home = home_dir("HOME is not set; cannot locate Claude local transcript roots")?;

    Ok(vec![
        home.join(".config").join("claude").join("projects"),
        home.join(".claude").join("projects"),
        home.join("Library")
            .join("Developer")
            .join("Xcode")
            .join("CodingAssistant")
            .join("ClaudeAgentConfig")
            .join("projects"),
    ])
}

pub fn read_cursor_access_token() -> io::Result<std::process::Output> {
    Command::new("security")
        .args([
            "find-generic-password",
            "-s",
            CURSOR_ACCESS_TOKEN_SERVICE,
            "-w",
        ])
        .stdin(Stdio::null())
        .output()
}

/// Builds the `curl` invocation for one dashboard method.
///
/// The request body is on the command line because it carries no secret; the
/// `Authorization` header is read from the config on stdin (`-K -`), so the
/// access token never reaches `argv`.
pub fn cursor_dashboard_request_command(method: &str, body: &str) -> Option<Command> {
    if !CURSOR_DASHBOARD_METHODS.contains(&method) {
        return None;
    }

    let mut command = Command::new("curl");
    command.args([
        "-sS",
        "-X",
        "POST",
        &format!("{CURSOR_DASHBOARD_URL_PREFIX}{method}"),
        "-K",
        "-",
        "-d",
        body,
    ]);
    Some(command)
}

pub fn allowed_cli_command_is_available(command: &str) -> bool {
    if !matches!(command, CLAUDE_CLI_COMMAND | CODEX_CLI_COMMAND) {
        return false;
    }

    Command::new(command)
        .arg("--version")
        .env("PATH", cli_process_path())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

pub fn is_allowed_external_url(url: &str) -> bool {
    ALLOWED_EXTERNAL_URLS.contains(&url)
}

#[cfg(target_os = "macos")]
pub fn open_external_url_with_system(url: &str) -> io::Result<()> {
    Command::new("open").arg(url).spawn()?.wait()?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn open_external_url_with_system(url: &str) -> io::Result<()> {
    Command::new("cmd")
        .args(["/C", "start", "", url])
        .spawn()?
        .wait()?;
    Ok(())
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn open_external_url_with_system(url: &str) -> io::Result<()> {
    Command::new("xdg-open").arg(url).spawn()?.wait()?;
    Ok(())
}

pub fn cli_process_path() -> OsString {
    let current_path = env::var_os("PATH").unwrap_or_default();
    let mut paths: Vec<_> = env::split_paths(&current_path).collect();

    let mut extra_paths: Vec<PathBuf> = vec![
        "/usr/local/bin".into(),
        "/usr/bin".into(),
        "/bin".into(),
        "/usr/sbin".into(),
        "/sbin".into(),
        "/opt/homebrew/bin".into(),
    ];

    if let Some(home) = env::var_os("HOME") {
        let home = PathBuf::from(home);
        extra_paths.push(home.join(".local").join("bin"));
        extra_paths.push(home.join(".cargo").join("bin"));
    }

    for path in extra_paths {
        if !paths.contains(&path) {
            paths.push(path);
        }
    }

    env::join_paths(paths).unwrap_or(current_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keychain_access_is_limited_to_cursor_access_token() {
        assert_eq!(CURSOR_ACCESS_TOKEN_SERVICE, "cursor-access-token");
    }

    #[test]
    fn network_access_is_limited_to_cursor_dashboard_api() {
        assert_eq!(
            CURSOR_DASHBOARD_URL_PREFIX,
            "https://api2.cursor.sh/aiserver.v1.DashboardService/"
        );
    }

    #[test]
    fn cursor_dashboard_methods_are_read_only_and_allowlisted() {
        for method in CURSOR_DASHBOARD_METHODS {
            assert!(method.starts_with("Get"), "{method} is not a Get method");
            assert!(cursor_dashboard_request_command(method, "{}").is_some());
        }

        assert!(cursor_dashboard_request_command("SetHardLimit", "{}").is_none());
        assert!(cursor_dashboard_request_command("GetTeamCustomerPortalUrl", "{}").is_none());
    }

    #[test]
    fn cli_availability_check_rejects_commands_outside_whitelist() {
        assert!(!allowed_cli_command_is_available("sh"));
        assert!(!allowed_cli_command_is_available("security"));
        assert!(!allowed_cli_command_is_available("curl"));
    }

    #[test]
    fn external_urls_are_limited_to_the_allowlist() {
        assert!(is_allowed_external_url(
            "https://code.claude.com/docs/en/setup"
        ));
        assert!(is_allowed_external_url(
            "https://developers.openai.com/codex/cli"
        ));
        assert!(!is_allowed_external_url("https://example.com"));
        assert!(is_allowed_external_url(
            "https://github.com/md2it/ai-limits"
        ));
        assert!(is_allowed_external_url(
            "https://github.com/md2it/ai-limits/blob/main/LICENSE"
        ));
    }

    #[test]
    fn a_path_inside_the_home_directory_is_shortened_to_a_tilde() {
        let home = Path::new("/Users/pat");

        assert_eq!(
            render_path(Path::new("/Users/pat/.codex"), Some(home)),
            "~/.codex"
        );
        assert_eq!(
            render_path(Path::new("/Users/pat/.claude/projects"), Some(home)),
            "~/.claude/projects"
        );
    }

    #[test]
    fn the_home_directory_itself_is_shortened_to_a_tilde() {
        assert_eq!(
            render_path(Path::new("/Users/pat"), Some(Path::new("/Users/pat"))),
            "~"
        );
    }

    #[test]
    fn a_path_outside_the_home_directory_stays_as_it_is() {
        let home = Path::new("/Users/pat");

        assert_eq!(
            render_path(Path::new("/tmp/.codex"), Some(home)),
            "/tmp/.codex"
        );
        assert_eq!(render_path(Path::new("/Users"), Some(home)), "/Users");
    }

    /// The home prefix is matched by path component, never by string prefix:
    /// a sibling directory must keep its own name.
    #[test]
    fn a_sibling_sharing_the_home_string_prefix_is_not_shortened() {
        let home = Path::new("/Users/pat");

        assert_eq!(
            render_path(Path::new("/Users/pat-backup/.codex"), Some(home)),
            "/Users/pat-backup/.codex"
        );
        assert_eq!(
            render_path(Path::new("/Users/patrick"), Some(home)),
            "/Users/patrick"
        );
    }

    #[test]
    fn an_unknown_home_directory_leaves_the_path_unchanged() {
        assert_eq!(
            render_path(Path::new("/Users/pat/.codex"), None),
            "/Users/pat/.codex"
        );
        assert_eq!(
            render_path(Path::new("/Users/pat/.codex"), Some(Path::new(""))),
            "/Users/pat/.codex"
        );
    }

    #[test]
    fn local_roots_match_documented_provider_paths() {
        let home = PathBuf::from(env::var_os("HOME").unwrap_or_else(|| "/tmp".into()));
        let roots = claude_local_roots().expect("HOME should be available in tests");
        assert!(roots.contains(&home.join(".config").join("claude").join("projects")));
        assert!(roots.contains(&home.join(".claude").join("projects")));
        assert!(roots.contains(
            &home
                .join("Library")
                .join("Developer")
                .join("Xcode")
                .join("CodingAssistant")
                .join("ClaudeAgentConfig")
                .join("projects")
        ));
    }
}
