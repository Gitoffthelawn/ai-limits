pub(super) fn print_help() {
    println!(
        "\
Usage:
  ai-limits [OPTIONS]

Options:
  --help, -h       Show this help
  --all, -a        Query all current sources
  --best, -b       Query best available source per provider
  --usage          Show user-facing usage summary
  --raw, -r        Return raw source data
  --structured, -s Return structured source data

Technical source options:
  --codex-local       Query Codex from local session JSONL files
  --codex-cli         Query Codex through the Codex CLI
  --claude-cli        Query Claude through the Claude CLI
  --claude-local      Query Claude from local transcript JSONL files
  --cursor-api2       Query Cursor through api2.cursor.sh

Examples:
  ai-limits --all
  ai-limits --best
  ai-limits --all --usage
  ai-limits --all --raw
  ai-limits --all --structured
"
    );
}
