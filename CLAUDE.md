# flux - AI-Guided Git Workflow Assistant

## AI Assistant Persona

**Philosophy:** Pragmatic, idiomatic, minimalist. Value working software over perfection. Write Rust that feels natural - use standard patterns, leverage type system, trust borrow checker.

**Code Style:** Idiomatic Rust (Result, ?, pattern matching), single responsibility, avoid premature abstraction (3+ uses), anyhow::Result for errors, clap derive macros, trust git2 directly.

**Mindset:** Test behavior not implementation, integration > unit tests, real-world testing, helpful error messages, local-first (zero network except Ollama).

**Anti-patterns:** Over-abstraction, premature optimization, feature creep, "clever" solutions, defensive programming, overengineered errors.

**Decision Framework:** (1) Does this solve the immediate problem? (2) Is this simplest solution? (3) Would Rust beginner understand in 6 months? (4) Does this add complexity?

**Remember:** Personal productivity tool, not production service. Make it work, make it right, make it fast - in that order.

---

## Overview

Profile-aware Git CLI enforcing workflow discipline through AI guardrails. Prevents mistakes (force-push to main, uncommitted work, wrong identity) before they happen. Context-switches between work/personal/opensource profiles.

**Target Users:** Developers with ADHD, junior developers, anyone context-switching between repos.

**Success Criteria (3 months):**
1. Zero protected branch violations (100+ operations)
2. 90% commit consistency (conventional format)
3. 3 active profiles in daily use

**NOT:** Git replacement, team tool, tutorial, cloud service, code review/CI/CD tool.

## MVP Features

1. **Profile System** - Create/switch profiles with git identity + workflow rules (stored `~/.config/flux/flux.toml`)
2. **Smart Commit** - AI conventional commits, profile-aware validation, interactive confirmation
3. **Branch Guardrails** - Block force-push to protected branches, checkout warnings
4. **Status** - Enhanced git status with profile, branch safety, staged changes
5. **Nexus Integration** - Auto-detect Nexus projects, extract sprint metadata for commits

## Tech Stack

**Core:** Rust 2021, clap (CLI), git2 (git ops), serde+toml (config), anyhow (errors), colored+dialoguer (UI), rustyline (REPL), reqwest (HTTP)
**LLM:** Ollama HTTP API (localhost:11434, Qwen 14B, temp=0, timeout=5s)
**Testing:** tempfile, assert_cmd, integration tests
**Storage:** TOML only (~/.config/flux/), no database, stateless

## Architecture

```
flux/src/
├── main.rs              # Entry point
├── commands/            # init, profile, commit, status, shell
├── config.rs            # Profile TOML
├── git.rs               # git2 ops
├── llm.rs               # Ollama client
├── context.rs           # Git+Nexus context
├── validator.rs         # Safety checks
└── nexus.rs             # Nexus detection
```

### Data Models

**Profile** (user_name, user_email, gpg_key, protected_branches, default_branch, rules, conventional_commits, auto_push)
**GitContext** (repo_path, current_branch, staged/unstaged/untracked_files, is_protected_branch, has_uncommitted_changes)
**NexusContext** (active_sprint, sprint_name, current_task, task_title, is_adhoc_mode)
**LLMRequest** (profile, git_context, nexus_context, operation_type, user_prompt)
**ValidationResult** (is_safe, violations, warnings, sanitized_output)
**ReplSession** (active_profile, working_directory, history_path, prompt_format)

### REPL Details

- `rustyline::Editor` with history (`~/.config/flux/history.txt`)
- Prompt: `flux({profile_name})>`
- Commands without `flux` prefix, special: exit/quit/help/clear
- Flow: parse → route to CLI handlers → maintain state → handle errors without exit
- Benefits: no startup overhead, state persistence, command history

## Key Workflows

**Init:** `flux init` → interactive prompts → save profile → activate
**Commit:** `flux commit` → gather context → LLM generates message → validate → confirm → execute
**Profile Switch:** `flux profile switch <name>` → update active profile
**REPL:** `flux shell` → interactive loop with profile in prompt
**Status:** `flux status` → show profile, branch safety, staged/unstaged changes

## Technical Decisions

**State:** Session-less (reload config), optional REPL in-memory state, no daemon
**LLM:** Sync HTTP to Ollama, structured prompts, fallback to manual if unavailable
**Safety:** Validate BEFORE execution, whitelist approach, confirm destructive ops, never execute raw LLM output
**Errors:** anyhow::Result, context chains, helpful messages, graceful degradation
**Nexus:** Optional via nexus.toml detection, read Obsidian vault metadata, fail gracefully

## Sprints

**0 (day 1):** Init repo, Cargo deps, CLI skeleton, test infra, verify Ollama
**1 (days 2-4):** Profile struct, TOML parse, flux init, profile commands, validation, tests
**1.5 (days 4-6):** REPL loop (rustyline), flux shell, command parse, prompt, history, exit/help/clear
**2 (days 7-9):** Git ops, context gather, validator, flux status, safety checks
**3 (days 10-13):** Ollama client, prompt builder, response parser, mocked tests, fallback
**4 (days 14-17):** flux commit with LLM, confirmation, profile rules, --all/--dry-run flags
**5 (days 18-20):** Nexus detection, config parse, sprint metadata, enhance prompts
**6 (days 21-23):** Integration tests, error messages, docs, real-world testing

**DoD:** Builds, tests pass, manual testing, conventional commit, retrospective, no regressions

## Future (NOT MVP)

Workflow automation, PR generation, git hooks, rebase assistance, stash labels, monorepo, commit analysis, team templates

## Environment

Fedora 43, VS Code + rust-analyzer + Claude Code, rustc 1.83+, git 2.43+, Ollama Qwen 2.5 14B

## Contributing

1. Read code before modifying
2. Use TodoWrite for multi-step tasks
3. Conventional commits
4. Test new features
5. Single responsibility
6. Minimal deps
7. Document public APIs
8. Test with real repos
9. Offline functionality
10. Privacy guarantees

---

**Related:** Nexus CLI (sprint/task management, Rust/TOML) - Flux reads metadata for commit context

**Status:** Planning phase, Sprint 0 ready
**Last Updated:** 2026-01-09
