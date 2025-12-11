# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**regelrecht-mvp** is an early-stage MVP project with two primary components:
- `engine/` - Core processing/business logic component
- `regulation/` - Regulatory compliance component

The repository is currently in initial setup phase with minimal code structure.

## Development Setup

This is a Python project managed with **uv** (https://github.com/astral-sh/uv).

### Initial Setup
```bash
# uv will automatically create a virtual environment and install dependencies
uv sync
```

### Just Commands (Required for Claude Code)

This project uses [just](https://github.com/casey/just) as a command runner.

**IMPORTANT FOR CLAUDE CODE:** All `just` commands have pre-authorized permissions configured in the project settings. This means Claude Code can run these commands **without asking the user for permission each time**. Always use `just` commands instead of raw `uv run` commands when available to avoid unnecessary permission prompts.

```bash
# List all available commands
just

# Run pytest
just test

# Run behave BDD tests
just behave

# Run all tests (pytest + behave)
just test-all

# Lint with ruff
just lint

# Format with ruff
just format

# Type check with ty
just typecheck

# Run pre-commit hooks
just pre-commit

# Sync dependencies
just sync
```

### Manual Commands
```bash
# Run Python scripts
uv run python script.py

# Add a new dependency
uv add package-name

# Add a development dependency
uv add --dev package-name
```

### Pre-commit Hooks
This repository uses pre-commit hooks for code quality:
- **Ruff**: Fast Python linter and formatter
- **ty**: Type checker (from Astral, same creators as Ruff)
- **Standard hooks**: Trailing whitespace, end-of-file fixer, YAML checks, etc.

Hooks are automatically installed with `uv run pre-commit install` and run on every commit.

**NEVER use `--no-verify` when committing.** Pre-commit hooks exist to catch issues early. If hooks fail, fix the underlying problem instead of bypassing them.

**No branding in commits.** Do not add "Generated with Claude Code" or "Co-Authored-By: Claude" lines to commit messages.

### Git Worktrees

When using git worktrees, create them **inside the project folder** (e.g., `.worktrees/`). This ensures Claude Code retains file access permissions without requiring re-authorization for each worktree.

```bash
# Create worktree inside project
git worktree add .worktrees/feature-branch feature-branch
```

### Type Checking
This project uses **ty** for Python type checking. All code must pass type checks.

```bash
# Run type checker
uv run ty check

# Type hints are enforced via pre-commit hooks
```

When adding type hints:
- Use modern Python 3.12+ syntax: `str | None` instead of `Optional[str]`
- Use `list[dict]` instead of `List[Dict]`
- Add `# type: ignore[rule-name]` comments sparingly for dynamic code

## Architecture Notes

### Directory Structure

- **engine/** - Article-based law execution engine
  - `article_loader.py` - Loads and parses article-based YAML laws
  - `uri_resolver.py` - Parses `regelrecht://` URIs
  - `rule_resolver.py` - Discovers and indexes laws by ID and output names
  - `context.py` - Execution context and value resolution
  - `engine.py` - Core article execution engine
  - `service.py` - Top-level law execution service
  - `utils.py` - Helper utilities

- **regulation/nl/** - Dutch legal regulations in machine-readable format
  - `wet/` - Formal laws (wetten)
  - `ministeriele_regeling/` - Ministerial regulations

- **features/** - Gherkin feature files for BDD testing

### Law Format

Laws are stored as article-based YAML files. **All law YAML files must conform to the official JSON schema** defined at:
- Schema: `https://raw.githubusercontent.com/MinBZK/poc-machine-law/refs/heads/main/schema/v0.2.0/schema.json`
- Validation: Use `uv run python script/validate.py <yaml_file>` to validate law files against the schema

The schema defines the required structure for machine-readable laws:

```yaml
$schema: https://raw.githubusercontent.com/MinBZK/poc-machine-law/refs/heads/main/schema/v0.2.0/schema.json
$id: "law_identifier"  # Slug for referencing (e.g., "zorgtoeslagwet")
regulatory_layer: "WET" | "MINISTERIELE_REGELING" | "AMVB"
publication_date: "YYYY-MM-DD"
bwb_id: "BWBRXXXXXXX"  # BWB identifier
url: "https://wetten.overheid.nl/..."  # Official government URL

articles:
  - number: "1"
    text: "Legal text in Dutch (verbatim from official source)..."
    url: "https://wetten.overheid.nl/...#Artikel1"

    machine_readable:
      definitions:
        CONSTANT_NAME:
          value: 123

      execution:
        parameters:
          - name: "BSN"
            type: "string"
            required: true

        input:
          - name: "INPUT_NAME"
            type: "number"
            source:
              regulation: "other_law"
              output: "output_name"  # Reference format: regulation + output
              parameters:
                BSN: "$BSN"

        output:
          - name: "OUTPUT_NAME"  # Output names make article publicly callable
            type: "boolean"

        actions:
          - output: "OUTPUT_NAME"
            operation: "EQUALS"
            subject: "$INPUT_NAME"
            value: 18
```

### Cross-Law References

Laws reference each other using `regelrecht://` URIs:

**Format:** `regelrecht://{law_id}/{output_name}#{field}`

**Example:** `regelrecht://zorgtoeslagwet/heeft_recht_op_zorgtoeslag#heeft_recht_op_zorgtoeslag`

The engine automatically resolves these URIs by:
1. Finding the law by `$id` slug
2. Finding the article by output name
3. Executing the article's logic
4. Extracting the requested `field` from outputs

## RFC Process

This project uses an RFC (Request for Comments) process for documenting significant design decisions and architectural choices.

- **Location**: `doc/rfcs/`
- **Process document**: See `doc/rfcs/RFC-000-rfc-process.md` for full details
- **Template**: Use `doc/rfcs/RFC-TEMPLATE.md` to create new RFCs

### When to Write an RFC

Write an RFC for:
- Law representation format changes
- Execution engine architecture changes
- Cross-cutting design patterns
- Integration patterns between components

### RFC Workflow

1. Copy `RFC-TEMPLATE.md` to `RFC-NNN-title.md` (increment number)
2. Fill in Context, Decision, and Why sections
3. Set Status to "Proposed"
4. Create PR for discussion
5. Update Status to "Accepted" once approved

## Code Reviews

After completing significant code changes (new features, refactors, bug fixes), proactively use the `code-reviewer` skill to review the changes before committing.

**Important:** Run the code review in a subagent using the Task tool with `subagent_type: "general-purpose"`. This keeps the review isolated and returns a summary to the main conversation.

Example prompt for the subagent:
```
Review the code changes in the current working directory using the code-reviewer skill.
Focus on: {specific areas if relevant}
Return: verdict, critical/important issues, and recommendations.
```

This ensures:
- Critical issues are caught before committing
- Reviews don't clutter the main conversation
- You get a structured summary of findings

## Technology Stack

- **Language**: Python 3.12+
- **Package Manager**: uv
- **Code Quality**: Ruff (linting and formatting), ty (type checking), pre-commit hooks

## Future Development

As this codebase grows, this CLAUDE.md should be updated to include:
- Architecture patterns and key design decisions
- Integration points between engine and regulation components
- Testing strategies and requirements
- API documentation and usage examples
