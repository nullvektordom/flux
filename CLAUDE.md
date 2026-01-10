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
├── commands/            # init, profile, commit, status, shell, flow, pr
├── config.rs            # Profile TOML
├── git.rs               # git2 ops
├── llm.rs               # Ollama client
├── context.rs           # Git+Nexus context
├── validator.rs         # Safety checks
├── nexus.rs             # Nexus detection
├── workflow.rs          # Flow orchestration (v2)
└── pr.rs                # PR generation (v2)
```

### Data Models

**MVP:**
- Profile (user_name, user_email, gpg_key, protected_branches, default_branch, rules, conventional_commits, auto_push)
- GitContext (repo_path, current_branch, staged/unstaged/untracked_files, is_protected_branch, has_uncommitted_changes)
- NexusContext (active_sprint, sprint_name, current_task, task_title, is_adhoc_mode)
- LLMRequest (profile, git_context, nexus_context, operation_type, user_prompt)
- ValidationResult (is_safe, violations, warnings, sanitized_output)
- ReplSession (active_profile, working_directory, history_path, prompt_format)

**V2:**
- WorkflowTemplate (name, branch_pattern, merge_strategy, tag_pattern, auto_push)
- WorkflowState (branch_name, steps_completed, current_step, can_rollback)
- WorkflowStep (step_type, status, output, timestamp)
- PullRequest (title, body, base_branch, head_branch, draft, labels)
- CommitAnalysis (commits_by_scope, breaking_changes, features, fixes)
- PrTemplate (sections, required_fields, custom_prompts)
- RemoteConfig (platform, api_token, repo_owner, repo_name)

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
**8 (days 21-23):** Integration tests, error messages, docs, real-world testing

**DoD:** Builds, tests pass, manual testing, conventional commit, retrospective, no regressions

## Version 2 Sprints

**6 (days 24-28):** Workflow orchestration - `flux flow` command, branch creation from task, auto-commit workflow, merge strategies, tag generation, profile-based flow templates, dry-run mode, rollback on failure

**7 (days 29-33):** PR automation - `flux pr` command, commit history analysis, LLM-generated PR title/body, changeset summarization, GitHub/GitLab API integration, link to Nexus tasks, template support, draft PR mode

### Sprint 6: Workflow Orchestration (days 24-28)

**Goal:** Automate complete feature branch workflow from creation to merge/tag

**Tasks:**
- Create `workflow.rs` module with orchestration logic
- Implement `flux flow start <task>` - creates branch from Nexus task or manual input
- Implement `flux flow finish` - commits, merges to default branch, creates tag
- Add workflow templates in profile config (feature/bugfix/hotfix patterns)
- Support merge strategies (merge commit, squash, rebase) per profile
- Implement tag generation with semantic versioning awareness
- Add `--dry-run` flag to preview all steps without execution
- Create rollback mechanism if any step fails
- Add workflow state tracking (~/.config/flux/workflow-state.json)
- Write integration tests for complete workflows
- Add REPL support for workflow commands

**Data Models:**
- `WorkflowTemplate` (name, branch_pattern, merge_strategy, tag_pattern, auto_push)
- `WorkflowState` (branch_name, steps_completed, current_step, can_rollback)
- `WorkflowStep` (step_type, status, output, timestamp)

**Exit Criteria:**
- Can run `flux flow start feature/auth` → creates branch, ready for commits
- Can run `flux flow finish` → commits all changes, merges to main, creates tag, pushes
- Validates safety at each step (protected branches, uncommitted work)
- Rollback works if merge fails
- Templates configurable per profile

**Commands:**
- `flux flow start [name]` - Start new workflow (create branch)
- `flux flow finish` - Complete workflow (commit → merge → tag → push)
- `flux flow abort` - Cancel workflow and return to previous branch
- `flux flow status` - Show current workflow state and next steps
- `flux flow templates` - List available workflow templates

### Sprint 7: PR Generation (days 29-33)

**Goal:** Generate pull requests with AI-analyzed commit history and changeset summaries

**Tasks:**
- Create `pr.rs` module with GitHub/GitLab API clients
- Implement `flux pr create` - analyzes commits since branch diverged
- Use LLM to generate PR title from conventional commits
- Use LLM to generate PR body with changeset summary
- Group commits by scope/type for organized PR description
- Extract linked issues/tickets from commit messages
- Support GitHub API (via `gh` CLI or direct API) for PR creation
- Support GitLab API as alternative
- Link to Nexus sprint/task if detected
- Add `--draft` flag for draft PRs
- Support PR templates from `.github/pull_request_template.md`
- Add `--base` flag to specify target branch (default from profile)
- Implement `flux pr update <number>` to regenerate description
- Write tests with mocked GitHub/GitLab responses

**Data Models:**
- `PullRequest` (title, body, base_branch, head_branch, draft, labels)
- `CommitAnalysis` (commits_by_scope, breaking_changes, features, fixes)
- `PrTemplate` (sections, required_fields, custom_prompts)
- `RemoteConfig` (platform, api_token, repo_owner, repo_name)

**Exit Criteria:**
- Can run `flux pr create` from feature branch → generates PR on GitHub
- PR title follows conventional format
- PR body has organized sections (Features, Fixes, Breaking Changes)
- Links to Nexus task if available
- Respects PR templates if present
- Works with both GitHub and GitLab
- Draft mode supported

**Commands:**
- `flux pr create` - Create PR from current branch with LLM-generated content
- `flux pr create --draft` - Create as draft PR
- `flux pr create --base develop` - Specify target branch
- `flux pr update <number>` - Regenerate PR description from commits
- `flux pr preview` - Show what PR would look like without creating

**Integration Notes:**
- Requires GitHub token in profile or `GITHUB_TOKEN` env var
- Falls back to `gh` CLI if installed
- For GitLab, use `GITLAB_TOKEN` and profile config
- Can be combined with workflow: `flux flow finish --pr` creates PR instead of merging

## Future (Version 3+)

Git hooks installer, rebase assistance, stash labels, monorepo support, commit analysis, team templates

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
