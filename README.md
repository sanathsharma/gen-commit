# Gen-Commit

A CLI tool that generates conventional commit messages using the Anthropic Claude AI model.

## Overview

Gen-Commit analyzes your staged git changes and generates meaningful, conventional commit messages. It leverages the Anthropic Claude API to understand code changes and produce well-formatted commit messages that follow the [Conventional Commits](https://www.conventionalcommits.org/) specification.

## Features

- Generates commit messages based on staged git changes
- Follows conventional commit format (`type(scope): description`)
- Considers branch name for context
- Supports Nx repository structure detection
- Allows custom scopes via a `scopes.txt` file
- Provides a dry-run option to preview messages without committing

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
# Set your Anthropic API key
export ANTHROPIC_API_KEY=your_api_key_here

# Stage your changes
git add .

# Generate a commit message
gen-commit

# Generate a commit message without committing (dry run)
gen-commit --dry-run

# Specify a different Claude model
gen-commit --model claude-3-opus-20240229

# Specify maximum token length for the response
gen-commit --max-tokens 1000
```

## Configuration

### API Key

Set your Anthropic API key as an environment variable:

```bash
export ANTHROPIC_API_KEY=your_api_key_here
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

## How It Works

1. Verifies you're in a git repository
2. Gets the current branch name for context
3. Checks for custom scopes and Nx repository structure
4. Retrieves the diff of staged changes
5. Sends the information to Anthropic's Claude AI model
6. Presents the generated commit message
7. Optionally commits with the generated message after confirmation

## Requirements

- Rust 1.56 or later
- Git
- Anthropic API key

## License

[Copyright (c) 2025 Sanath Sharma](LICENSE)
