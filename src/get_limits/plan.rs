use crate::types::Source;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourcePlan {
    Single(Source),
    Chain {
        label: &'static str,
        sources: &'static [Source],
    },
}

impl SourcePlan {
    pub fn label(self) -> &'static str {
        match self {
            Self::Single(source) => source.label(),
            Self::Chain { label, .. } => label,
        }
    }
}

const FAST_CODEX_CHAIN: &[Source] = &[Source::CodexLocal];
const FAST_CLAUDE_CHAIN: &[Source] = &[Source::ClaudeLocal];
const FAST_CURSOR_CHAIN: &[Source] = &[Source::CursorApi2];

const CLI_FALLBACK_CODEX_CHAIN: &[Source] = &[Source::CodexLocal, Source::CodexRpc];
const CLI_FALLBACK_CLAUDE_CHAIN: &[Source] = &[Source::ClaudeLocal, Source::ClaudeRpc];
const CLI_FALLBACK_CURSOR_CHAIN: &[Source] = &[Source::CursorApi2];

const CLI_FIRST_CODEX_CHAIN: &[Source] = &[Source::CodexRpc, Source::CodexLocal];
const CLI_FIRST_CLAUDE_CHAIN: &[Source] = &[Source::ClaudeRpc, Source::ClaudeLocal];
const CLI_FIRST_CURSOR_CHAIN: &[Source] = &[Source::CursorApi2];

pub fn default_source_plan() -> Vec<SourcePlan> {
    fast_free_source_plan()
}

pub fn best_source_plan() -> Vec<SourcePlan> {
    cli_fallback_source_plan()
}

pub fn fast_free_source_plan() -> Vec<SourcePlan> {
    vec![
        SourcePlan::Chain {
            label: "codex",
            sources: FAST_CODEX_CHAIN,
        },
        SourcePlan::Chain {
            label: "claude",
            sources: FAST_CLAUDE_CHAIN,
        },
        SourcePlan::Chain {
            label: "cursor",
            sources: FAST_CURSOR_CHAIN,
        },
    ]
}

pub fn cli_fallback_source_plan() -> Vec<SourcePlan> {
    vec![
        SourcePlan::Chain {
            label: "codex",
            sources: CLI_FALLBACK_CODEX_CHAIN,
        },
        SourcePlan::Chain {
            label: "claude",
            sources: CLI_FALLBACK_CLAUDE_CHAIN,
        },
        SourcePlan::Chain {
            label: "cursor",
            sources: CLI_FALLBACK_CURSOR_CHAIN,
        },
    ]
}

pub fn cli_first_source_plan() -> Vec<SourcePlan> {
    vec![
        SourcePlan::Chain {
            label: "codex",
            sources: CLI_FIRST_CODEX_CHAIN,
        },
        SourcePlan::Chain {
            label: "claude",
            sources: CLI_FIRST_CLAUDE_CHAIN,
        },
        SourcePlan::Chain {
            label: "cursor",
            sources: CLI_FIRST_CURSOR_CHAIN,
        },
    ]
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiSourcePlanOptions {
    pub enabled_codex: bool,
    pub enabled_claude: bool,
    pub enabled_cursor: bool,
}

impl Default for UiSourcePlanOptions {
    fn default() -> Self {
        Self {
            enabled_codex: true,
            enabled_claude: true,
            enabled_cursor: true,
        }
    }
}

// The desktop app always queries the CLI-first (Best) chain: it used to let
// users trade freshness for speed via a Fast/Full/Best setting, but RPC made
// the CLI-backed sources fast enough that the tradeoff no longer bought
// anything, so the choice was removed and this always resolves to Best.
pub fn ui_source_plan(options: UiSourcePlanOptions) -> Vec<SourcePlan> {
    cli_first_source_plan()
        .into_iter()
        .filter(|plan| match plan.label() {
            "codex" => options.enabled_codex,
            "claude" => options.enabled_claude,
            "cursor" => options.enabled_cursor,
            _ => false,
        })
        .collect()
}

pub fn source_list_plan(sources: Vec<Source>) -> Vec<SourcePlan> {
    sources.into_iter().map(SourcePlan::Single).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_plan_uses_fast_free_provider_chains() {
        assert_eq!(
            default_source_plan(),
            vec![
                SourcePlan::Chain {
                    label: "codex",
                    sources: FAST_CODEX_CHAIN
                },
                SourcePlan::Chain {
                    label: "claude",
                    sources: FAST_CLAUDE_CHAIN
                },
                SourcePlan::Chain {
                    label: "cursor",
                    sources: FAST_CURSOR_CHAIN
                },
            ]
        );
    }

    #[test]
    fn best_plan_adds_cli_fallbacks_for_codex_and_claude() {
        assert_eq!(
            best_source_plan(),
            vec![
                SourcePlan::Chain {
                    label: "codex",
                    sources: CLI_FALLBACK_CODEX_CHAIN
                },
                SourcePlan::Chain {
                    label: "claude",
                    sources: CLI_FALLBACK_CLAUDE_CHAIN
                },
                SourcePlan::Chain {
                    label: "cursor",
                    sources: CLI_FALLBACK_CURSOR_CHAIN
                },
            ]
        );
    }

    #[test]
    fn cli_first_plan_prefers_cli_for_codex_and_claude() {
        assert_eq!(
            cli_first_source_plan(),
            vec![
                SourcePlan::Chain {
                    label: "codex",
                    sources: CLI_FIRST_CODEX_CHAIN
                },
                SourcePlan::Chain {
                    label: "claude",
                    sources: CLI_FIRST_CLAUDE_CHAIN
                },
                SourcePlan::Chain {
                    label: "cursor",
                    sources: CLI_FIRST_CURSOR_CHAIN
                },
            ]
        );
    }

    #[test]
    fn ui_source_plan_defaults_to_cli_first_chains() {
        assert_eq!(
            ui_source_plan(UiSourcePlanOptions::default()),
            cli_first_source_plan()
        );
    }

    #[test]
    fn ui_source_plan_filters_disabled_providers() {
        assert_eq!(
            ui_source_plan(UiSourcePlanOptions {
                enabled_codex: true,
                enabled_claude: false,
                enabled_cursor: true,
            }),
            vec![
                SourcePlan::Chain {
                    label: "codex",
                    sources: CLI_FIRST_CODEX_CHAIN
                },
                SourcePlan::Chain {
                    label: "cursor",
                    sources: CLI_FIRST_CURSOR_CHAIN
                },
            ]
        );
    }
}
