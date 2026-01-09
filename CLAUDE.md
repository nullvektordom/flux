# Flux - AI-Guided Git Workflow Assistant

## AI Assistant Persona

When working on this project, adopt the following approach:

**Philosophy:** Pragmatic, idiomatic, and minimalist. Value working software over perfect software. Prioritize simplicity and clarity over cleverness. Write Rust that feels natural to experienced Rustaceans - use standard patterns, leverage the type system, and trust the borrow checker.

**Code Style:**
- Write idiomatic Rust: use `Result` types, `?` operator, pattern matching
- Prefer explicit over implicit, but don't over-engineer
- Single responsibility: functions and modules do one thing well
- Avoid premature abstractions - three uses before extracting
- Use `anyhow::Result` for application errors, custom types only when needed
- Leverage `clap` derive macros instead of builder pattern
- Trust `git2` - don't wrap everything, use it directly where it makes sense

**Development Mindset:**
- Test behavior, not implementation details
- Integration tests over unit tests for CLI tools
- Real-world testing beats 100% coverage
- Error messages should help users fix problems, not just report failures
- Performance matters but correctness comes first
- Local-first means ZERO network calls except to localhost Ollama

**Anti-patterns to AVOID:**
- Over-abstraction: Don't create trait hierarchies for 2 implementations
- Premature optimization: Get it working, then make it fast if needed
- Feature creep: MVP means minimum, stick to the plan
- Magic: No DSLs, no complex macros, no "clever" solutions
- Defensive programming: Don't validate internal invariants, use types
- Overengineering error handling: Not every error needs a custom type

**Decision Framework:**
1. Does this solve the immediate problem? (If no, don't build it)
2. Is this the simplest solution that works? (If no, simplify)
3. Would a Rust beginner understand this in 6 months? (If no, refactor)
4. Does this add dependencies or complexity? (If yes, justify it)

**When stuck:** Read existing Rust CLI tools (ripgrep, fd, bat) for inspiration. Copy patterns from the ecosystem. Don't reinvent wheels.

**Remember:** This is a personal productivity tool, not a production service. Pragmatism and shipping beats architectural purity. Make it work, make it right, make it fast - in that order.

---

## Project Overview

Flux is a profile-aware Git CLI that enforces workflow discipline through AI-guided guardrails, preventing mistakes before they happen while adapting to different project contexts. Built for developers with ADHD, junior developers learning git workflows, and anyone who context-switches between repos with different branch strategies.

## Core Problem

Developers struggle to maintain git discipline across different contexts - forgetting to checkout branches, committing files that shouldn't be committed, or accidentally force-pushing to protected branches. This is especially challenging when switching between work/personal/opensource projects with different conventions.

## Solution

A local-first, privacy-focused Git CLI that uses local LLM (Qwen 14B via Ollama) to provide intelligent commit messages and enforce safety guardrails based on user-defined profiles. Each profile contains git identity, workflow rules, and branch protection settings.

## Success Criteria (3 months)

1. **Zero protected branch violations** - User successfully prevented from force-pushing to main/develop across 100+ git operations
2. **90% commit consistency** - 9 out of 10 commits follow conventional format without manual correction
3. **3 active profiles in production use** - User maintains separate work/personal/opensource profiles with different rules, switching seamlessly between contexts

## What This Is NOT

- Not a git replacement or wrapper around every git command
- Not a team collaboration tool (single-user focused)
- Not a git learning tutorial or documentation system
- Not cloud-based or SaaS requiring internet connectivity
- Not opinionated about project structure (works with any repo)
- Not a code review or CI/CD tool

## MVP Features

### 1. Profile System
- Create/switch between profiles (work/personal/opensource)
- Different git identities and rules per profile
- Stored in `~/.config/flux/flux.toml`

### 2. Smart Commit
- AI-assisted conventional commit messages
- Profile-aware validation and safety checks
- Interactive confirmation with preview

### 3. Branch Guardrails
- Prevent operations on protected branches
- No force-push to main/develop
- Checkout warnings for protected branches

### 4. Status Awareness
- Enhanced git status showing current profile
- Branch safety indicators
- Staged changes overview

### 5. Nexus Integration
- Auto-detect Nexus projects
- Extract commit context from sprint metadata
- Enhance commit messages with sprint scope

## Tech Stack

- **Language:** Rust (edition 2021)
- **CLI Framework:** clap (v4.5) with derive macros
- **Git Operations:** git2 (v0.18) - libgit2 bindings
- **Config:** TOML files (serde + toml crates)
- **LLM Interface:** Ollama HTTP API (local Qwen 14B)
- **Error Handling:** anyhow (v1.0)
- **Terminal UI:** colored (v2.1) + dialoguer (v0.11)
- **HTTP Client:** reqwest (v0.11)

### Key Dependencies

1. `clap` - CLI argument parsing
2. `git2` - Git operations
3. `anyhow` - Error handling with context
4. `serde` + `toml` - Config serialization
5. `colored` - Terminal colors
6. `dialoguer` - Interactive prompts
7. `reqwest` - Ollama API client
8. `dirs` - Cross-platform config paths
9. `chrono` - Timestamps
10. `tempfile` + `assert_cmd` - Testing

## Architecture

### Project Structure

```
flux/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── commands/            # Command implementations
│   │   ├── mod.rs
│   │   ├── init.rs          # flux init
│   │   ├── profile.rs       # flux profile
│   │   ├── commit.rs        # flux commit
│   │   ├── status.rs        # flux status
│   │   └── shell.rs         # flux shell (REPL)
│   ├── config.rs            # Profile config (TOML)
│   ├── git.rs               # Git operations via git2
│   ├── llm.rs               # Ollama client
│   ├── context.rs           # Context gathering
│   ├── validator.rs         # Safety checks
│   └── nexus.rs             # Nexus project detection
├── tests/
│   ├── integration/         # End-to-end tests
│   └── fixtures/            # Test data
├── templates/
│   └── flux.toml.example    # Example config
└── docs/
    └── PROFILES.md          # Profile guide
```

### Data Model

#### Profile
User's git identity and workflow rules for a specific context. Stored in `~/.config/flux/flux.toml`.

**Fields:**
- `name` - Profile identifier
- `user_name` - Git user.name
- `user_email` - Git user.email
- `gpg_signing` - Enable GPG signing
- `gpg_key` - GPG key ID
- `protected_branches` - List of protected branches
- `default_branch` - Default branch name
- `rules` - Natural language workflow rules
- `conventional_commits` - Enforce conventional commits
- `auto_push` - Auto-push after commit

#### GitContext
Snapshot of current repository state gathered before operations.

**Fields:**
- `repo_path` - Path to repository
- `current_branch` - Active branch name
- `staged_files` - Files in staging area
- `unstaged_files` - Modified files
- `untracked_files` - New files
- `is_protected_branch` - Safety flag
- `has_uncommitted_changes` - Dirty flag

#### NexusContext
Optional metadata from Nexus project if detected.

**Fields:**
- `active_sprint` - Current sprint number
- `sprint_name` - Sprint identifier
- `current_task` - Active task ID
- `task_title` - Task description
- `is_adhoc_mode` - Outside sprint work

#### LLMRequest
Structured prompt sent to local LLM.

**Fields:**
- `profile` - Active profile
- `git_context` - Repository state
- `nexus_context` - Sprint metadata
- `operation_type` - commit/branch/merge
- `user_prompt` - User input

#### ValidationResult
Safety check result after LLM output.

**Fields:**
- `is_safe` - Validation passed
- `violations` - Rule breaches
- `warnings` - Non-blocking issues
- `sanitized_output` - Cleaned output

## User Workflows

### First-Time Setup

```bash
$ flux init
Welcome to Flux! Let's set up your first profile.

Profile name: work
Git name: John Doe
Git email: john.doe@company.com
Enable GPG signing? yes
GPG key ID: ABC123DEF456
Protected branches (comma-separated): main,develop
Default branch: main

Define workflow rules (plain text):
> Always use conventional commits
> Require ticket number in commit message
> Never force-push to protected branches

✓ Profile 'work' created and activated!
```

### Daily Commit Workflow

```bash
$ flux commit
[Gathering context...]
Branch: feature/add-login
Staged: 3 files
Profile: work

[Contacting local LLM...]
Generated commit message:

feat(auth): implement JWT-based login system

- Add JWT token generation with RS256
- Implement login endpoint with validation
- Add refresh token rotation

Refs: TICKET-123

✓ Commit? [y/n/e(dit)/r(egenerate)]
```

### Profile Switching

```bash
$ flux profile list
Available profiles:
  * work (active)
    personal
    opensource

$ flux profile switch personal
✓ Switched to profile 'personal'
```

### Enhanced Status

```bash
$ flux status
Profile: work
Branch: feature/add-login (safe)
Protected branches: main, develop

Staged changes:
  M src/auth/jwt.rs
  M src/api/login.rs
  A tests/auth_test.rs

Unstaged changes:
  M README.md
```

### Safety Guardrail

```bash
$ git push --force origin main
✗ ERROR: Cannot force-push to protected branch 'main'

Protected branches in 'work' profile: main, develop
Suggestion: Use 'git push' without --force, or push to a feature branch
```

## Critical Technical Decisions

### State Management
- Session-less by default (each command loads config fresh)
- Optional REPL mode maintains in-memory state
- No persistent daemon process

### LLM Integration
- Synchronous HTTP calls to Ollama API (localhost:11434)
- Temperature=0 for consistency
- Timeout=5s with fallback to manual mode
- Structured prompts with context injection
- Response parsing with fallback to user input if malformed

### Safety Architecture
- Validator runs BEFORE execution (never after)
- Whitelist approach (only known-safe operations allowed)
- User confirmation required for destructive actions
- Protected branch list enforced at validation layer
- LLM output never executed raw

### Data Persistence
- TOML config files only (no database)
- Profile data in `~/.config/flux/flux.toml`
- No command history/logs (use git log)
- Stateless commands for simplicity

### Error Handling
- `anyhow::Result` throughout
- Context chains for debugging
- User-friendly error messages with suggestions
- Graceful degradation (LLM fails → manual mode)

### Nexus Integration
- Detection via `nexus.toml` presence check
- Optional enhancement (works without Nexus)
- Reads sprint metadata from Obsidian vault path in config
- Fails gracefully if Obsidian files missing

## Development Roadmap

### Sprint 0: Setup (day 1)
- Initialize Flux repo with Nexus
- Setup Cargo.toml with dependencies
- Create basic CLI skeleton
- Setup test infrastructure
- Verify Ollama running locally

### Sprint 1: Profile System (days 2-4)
- Implement Profile struct and TOML parsing
- Create `flux init` command
- Implement `flux profile` commands
- Add profile validation
- Write config tests

### Sprint 2: Git Context + Safety (days 5-7)
- Implement git operations
- Create context gathering
- Build validator with branch protection
- Implement `flux status`
- Add safety checks

### Sprint 3: LLM Integration (days 8-11)
- Create Ollama HTTP client
- Implement prompt builder
- Add response parser
- Write tests with mocked responses
- Add graceful degradation

### Sprint 4: Smart Commit (days 12-15)
- Implement `flux commit` with LLM
- Add interactive confirmation
- Integrate profile rules
- Add `--all` and `--dry-run` flags
- Execute commits with git2

### Sprint 5: Nexus Integration (days 16-18)
- Create project detection
- Parse Nexus config
- Read sprint metadata
- Enhance LLM prompts
- Add Nexus tests

### Sprint 6: Polish & Testing (days 19-21)
- Comprehensive integration tests
- Improve error messages
- Complete documentation
- Real-world testing
- Ready for daily use

## Definition of Done (Each Sprint)

- [ ] Builds without errors (`cargo build`)
- [ ] All tests pass (`cargo test`)
- [ ] Tested manually with real git repo
- [ ] Code committed with conventional message
- [ ] Sprint retrospective documented
- [ ] No regressions in previous features

## Future (NOT MVP)

- Workflow automation (branch → commit → merge → tag)
- PR generation from commit history
- Git hooks installer
- Interactive rebase assistance
- Stash management with semantic labels
- Multi-repo operations (monorepo support)
- Commit history analysis
- Team profile templates

## Development Environment

- **IDE:** VS Code with rust-analyzer + Claude Code
- **OS:** Linux (Fedora 43, kernel 6.17.12)
- **Device:** Desktop workstation
- **Rust:** stable (rustc 1.83+)
- **Git:** 2.43+
- **LLM:** Ollama with Qwen 2.5 14B model

## Project Constraints

- **Budget:** Time-only (personal project)
- **Deadline:** 3 months to MVP
- **Platform:** Linux/Mac desktop CLI
- **Storage:** Local only (~/.config/flux/)
- **LLM:** Must work offline with local models
- **Privacy:** No cloud dependency, all data local

## Getting Started

1. Ensure Ollama is running with Qwen 2.5 14B model
2. Install Flux (will be available via cargo or binary)
3. Run `flux init` to create first profile
4. Navigate to git repo
5. Use `flux commit` instead of `git commit`
6. Switch profiles with `flux profile switch`

## Contributing Guidelines

When working on Flux:

1. **Always read existing code before modifying**
2. **Use TodoWrite to track multi-step tasks**
3. **Follow conventional commits** (we eat our own dogfood)
4. **Write tests for new features**
5. **Maintain single-responsibility principle**
6. **Keep dependencies minimal**
7. **Document public APIs**
8. **Test with real git repos**
9. **Ensure offline functionality**
10. **Preserve privacy guarantees**

## Related Projects

- **Nexus CLI:** Sprint planning and task management tool (Rust, TOML-based)
- Integration point: Flux reads Nexus metadata to enhance commit context

---

**Last Updated:** 2026-01-09
**Status:** Planning phase - Sprint 0 ready to start
**Next Step:** Initialize Cargo project and implement basic CLI skeleton
