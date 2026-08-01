#!/bin/sh
# Run all automated quality checks manually with: npm run quality
set -eu

project_root="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
cd "$project_root"

printf '%s\n' 'Checking frontend ES modules...'
node scripts/check-frontend-modules.mjs
printf '%s\n' 'Checking Markdown links...'
node scripts/check-markdown-links.mjs
printf '%s\n' 'Checking Rust formatting...'
cargo fmt --all -- --check
printf '%s\n' 'Linting Rust...'
cargo clippy --workspace --all-targets -- -D warnings
printf '%s\n' 'Running Rust tests...'
cargo test --workspace --all-targets
