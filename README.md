# Gen-Commit

A CLI tool that generates conventional commit messages using AI models from Anthropic and OpenAI.

## Overview

Gen-Commit analyzes your staged git changes and generates meaningful, conventional commit messages. It leverages AI models from Anthropic and OpenAI to understand code changes and produce well-formatted commit messages that follow the [Conventional Commits](https://www.conventionalcommits.org/) specification.

## Features

- Generates commit messages based on staged git changes
- Follows conventional commit format (`type(scope): description`)
- Supports both Anthropic and OpenAI models
- AI-powered change analysis for enhanced commit message context
- Optional `--no-analysis` mode for faster, cost-effective commits
- Considers branch name for context
- Supports Nx repository structure detection
- Allows custom scopes via a `scopes.txt` file
- Provides a dry-run option to preview messages without committing
- Supports ignoring specific files or directories from the git diff analysis

## Installation

```bash
# Clone the repository
git clone https://github.com/sanathsharma/gen-commit.git
cd gen-commit

# Build the project
cargo build --release

# Optional: Move the binary to your PATH
cp target/release/gen-commit ~/.local/bin/
```

## Usage

```bash
# Set your API key for the model provider you want to use
export ANTHROPIC_API_KEY=your_api_key_here
# OR
export OPENAI_API_KEY=your_api_key_here

# Stage your changes
git add .

# Generate a commit message
gen-commit

# Generate a commit message without committing (dry run)
gen-commit --dry-run

# Specify a different model with provider prefix
gen-commit --model anthropic:claude-sonnet-4-20250514
gen-commit --model openai:gpt-4.1-mini

# Specify maximum token length for the response
gen-commit --max-tokens 1000

# Ignore specific files or directories in the diff
gen-commit --ignore "node_modules,dist,*.log"

# Skip AI-powered change analysis for faster, cost-effective commits
gen-commit --no-analysis
```

## Configuration

### API Keys

Set your API key as an environment variable for the model provider you want to use:

```bash
# For Anthropic models
export ANTHROPIC_API_KEY=your_api_key_here

# For OpenAI models
export OPENAI_API_KEY=your_api_key_here
```

### Default Model

You can set a default model by setting the `GC_DEFAULT_MODEL` environment variable:

```bash
export GC_DEFAULT_MODEL=openai:gpt-4
```

### Custom Scopes

Create a `scopes.txt` file in your repository root with one scope per line:

```
api
ui
core
docs
ci
```

### Ignore List

You can specify files or directories to ignore when generating commit messages:

```bash
# Ignore a single file
gen-commit --ignore "package-lock.json"

# Ignore multiple files or directories (comma-separated)
gen-commit --ignore "package-lock.json,node_modules,dist"

# Use glob patterns
gen-commit --ignore "package-lock.json,*.log"
```

By default, `package-lock.json,Cargo.lock,bun.lock,pnpm-lock.yaml` are ignored.

### Change Default Ignore List

You can change the default ignore list by setting the `GC_IGNORE_LIST` environment variable:

```bash
export GC_IGNORE_LIST=package-lock.json,*.log
```

### Analysis Mode

By default, gen-commit uses a two-step AI approach for enhanced commit messages:

1. **Change Analysis**: AI analyzes the git diff to understand what changed
2. **Commit Generation**: AI generates a commit message using the analysis + context

#### Skip Analysis Mode (`--no-analysis`)

Use `--no-analysis` to skip the change analysis step and generate commit messages directly from the git diff:

```bash
gen-commit --no-analysis
```

**Benefits:**
- **~50% cost reduction** - Uses half the AI tokens by skipping analysis
- **Faster execution** - One fewer API call means quicker results  
- **Good for simple changes** - Direct diff-to-commit works well for straightforward modifications

**When to use:**
- Simple bug fixes or refactoring
- Cost-conscious workflows
- When speed is prioritized over detailed analysis
- Changes where git diff provides sufficient context

## How It Works

**Default Mode (with analysis):**
1. Verifies you're in a git repository
2. Gets the current branch name for context
3. Checks for custom scopes and Nx repository structure
4. Retrieves the diff of staged changes
5. **Analyzes changes** with AI to understand modifications
6. **Generates commit message** using analysis + context
7. Presents the generated commit message with token usage
8. Optionally commits with the generated message after confirmation

**Skip Analysis Mode (`--no-analysis`):**
1. Verifies you're in a git repository  
2. Gets the current branch name for context
3. Checks for custom scopes and Nx repository structure
4. Retrieves the diff of staged changes
5. **Generates commit message** directly from diff + context *(skips analysis step)*
6. Presents the generated commit message with token usage
7. Optionally commits with the generated message after confirmation

## Requirements

- Rust 1.56 or later
- Git
- API key for Anthropic or OpenAI

## License

[Copyright (c) 2025 Sanath Sharma](LICENSE)
