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

const CLI_FALLBACK_CODEX_CHAIN: &[Source] = &[Source::CodexLocal, Source::CodexCli];
const CLI_FALLBACK_CLAUDE_CHAIN: &[Source] = &[Source::ClaudeLocal, Source::ClaudeCli];
const CLI_FALLBACK_CURSOR_CHAIN: &[Source] = &[Source::CursorApi2];

const CLI_FIRST_CODEX_CHAIN: &[Source] = &[Source::CodexCli, Source::CodexLocal];
const CLI_FIRST_CLAUDE_CHAIN: &[Source] = &[Source::ClaudeCli, Source::ClaudeLocal];
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourcePriority {
    Fast,
    #[default]
    Full,
    Best,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiSourcePlanOptions {
    pub enabled_codex: bool,
    pub enabled_claude: bool,
    pub enabled_cursor: bool,
    pub source_priority: SourcePriority,
}

impl Default for UiSourcePlanOptions {
    fn default() -> Self {
        Self {
            enabled_codex: true,
            enabled_claude: true,
            enabled_cursor: true,
            source_priority: SourcePriority::Full,
        }
    }
}

pub fn ui_source_plan(options: UiSourcePlanOptions) -> Vec<SourcePlan> {
    let plans = match options.source_priority {
        SourcePriority::Fast => fast_free_source_plan(),
        SourcePriority::Full => cli_fallback_source_plan(),
        SourcePriority::Best => cli_first_source_plan(),
    };

    plans
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
    fn ui_source_plan_defaults_to_full_priority() {
        assert_eq!(
            UiSourcePlanOptions::default().source_priority,
            SourcePriority::Full
        );
        assert_eq!(
            ui_source_plan(UiSourcePlanOptions::default()),
            cli_fallback_source_plan()
        );
    }

    #[test]
    fn ui_source_plan_filters_disabled_providers() {
        assert_eq!(
            ui_source_plan(UiSourcePlanOptions {
                enabled_codex: true,
                enabled_claude: false,
                enabled_cursor: true,
                source_priority: SourcePriority::Fast,
            }),
            vec![
                SourcePlan::Chain {
                    label: "codex",
                    sources: FAST_CODEX_CHAIN
                },
                SourcePlan::Chain {
                    label: "cursor",
                    sources: FAST_CURSOR_CHAIN
                },
            ]
        );
    }

    #[test]
    fn ui_source_plan_uses_cli_fallback_chains_for_full_priority() {
        assert_eq!(
            ui_source_plan(UiSourcePlanOptions {
                enabled_codex: true,
                enabled_claude: true,
                enabled_cursor: false,
                source_priority: SourcePriority::Full,
            }),
            vec![
                SourcePlan::Chain {
                    label: "codex",
                    sources: CLI_FALLBACK_CODEX_CHAIN
                },
                SourcePlan::Chain {
                    label: "claude",
                    sources: CLI_FALLBACK_CLAUDE_CHAIN
                },
            ]
        );
    }

    #[test]
    fn ui_source_plan_uses_cli_first_chains_for_best_priority() {
        assert_eq!(
            ui_source_plan(UiSourcePlanOptions {
                enabled_codex: true,
                enabled_claude: true,
                enabled_cursor: false,
                source_priority: SourcePriority::Best,
            }),
            vec![
                SourcePlan::Chain {
                    label: "codex",
                    sources: CLI_FIRST_CODEX_CHAIN
                },
                SourcePlan::Chain {
                    label: "claude",
                    sources: CLI_FIRST_CLAUDE_CHAIN
                },
            ]
        );
    }
}
