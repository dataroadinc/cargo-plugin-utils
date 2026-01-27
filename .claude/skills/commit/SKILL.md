---
name: commit
description:
  Create commits following Angular Conventional Commits format with
  proper scope naming for consistent changelog generation
---

# Commit Skill

Use this skill when creating git commits. All commits must follow
Angular Conventional Commits format because:

1. Git hooks enforce it (commit will be rejected otherwise)
2. Cocogitto parses commits to generate release changelogs

## Commit Message Format

```text
<type>(<scope>): <subject>

[optional body]

[optional footer]
```

**Requirements:**

- Type: lowercase, from allowed list
- Scope: mandatory, lowercase, describes what changed
- Subject: lowercase start, imperative mood, no period
- Breaking changes: add exclamation mark before colon (e.g. feat(api)!:)

## Allowed Types

### Angular standard types

| Type       | Purpose                          | In Changelog |
| ---------- | -------------------------------- | ------------ |
| `feat`     | New feature                      | Yes          |
| `fix`      | Bug fix                          | Yes          |
| `docs`     | Documentation only               | Yes          |
| `refactor` | Code change (no feat/fix)        | Yes          |
| `perf`     | Performance improvement          | Yes          |
| `build`    | Build system changes             | Yes          |
| `style`    | Formatting, whitespace           | No           |
| `test`     | Adding/fixing tests              | No           |
| `ci`       | CI/CD configuration              | No           |

### Extended types (not in original Angular spec)

These are widely adopted extensions configured in `cog.toml`:

| Type       | Purpose                          | In Changelog |
| ---------- | -------------------------------- | ------------ |
| `chore`    | Maintenance, deps, tooling       | No           |
| `revert`   | Revert previous commit           | Yes          |

## Scope Naming Guidelines

Scopes should be consistent to group related changes in changelogs.

### Component scopes

- `lib` - main library exports (src/lib.rs)
- `logger` - logging utilities
- `progress` - progress bar/spinner utilities
- `scrolling` - scrolling text utilities
- `tty` - terminal detection utilities
- `common` - shared utility functions

### Infrastructure scopes

- `version` - version bumps
- `deps` - dependency updates
- `ci` - CI/CD workflows (use with `ci:` or `chore:` type)
- `config` - configuration files
- `msrv` - minimum supported Rust version

### Documentation scopes

- `readme` - README.md changes
- `claude` - CLAUDE.md, Claude Code skills

### Testing scopes

- `tests` - general test changes

## Examples

### Good commit messages

```text
feat(logger): add colored output support
fix(progress): handle terminal resize correctly
docs(readme): update installation instructions
refactor(scrolling): extract scroll buffer logic
test(tty): add tests for terminal detection
chore(deps): update indicatif to 0.18
ci(workflows): add release binary uploads
feat(lib)!: change public API structure
```

### Bad commit messages

```text
feat: add feature              # missing scope
Fix(lib): Fix bug              # uppercase type and subject
feat(lib): Add feature.        # uppercase subject, has period
updated the readme             # wrong format entirely
feat(lib) add feature          # missing colon
feat(Lib): add feature         # uppercase scope
```

## Breaking Changes

For breaking changes, add an exclamation mark after the scope:

```text
feat(logger)!: change logger initialization API

BREAKING CHANGE: Logger::new() now requires a config parameter.
Use Logger::default() for the old behavior.
```

This will appear prominently in the changelog.

## Commit Process

1. Stage your changes: `git add <files>`
2. Commit with message: `git commit -m "type(scope): subject"`
3. Hooks automatically run:
   - pre-commit: fmt and clippy checks
   - commit-msg: validates conventional commit format
   - post-commit: verifies signature

If the commit is rejected, fix the issue and try again.

## Multi-line Commits

For complex changes, use a body:

```bash
git commit -m "feat(progress): add multi-bar support

Allows displaying multiple progress bars simultaneously.
Useful for parallel operations like downloads.

Closes #42"
```

## Checking Recent Commits

To see the commit style used in this repo:

```bash
git log --oneline -20
```
