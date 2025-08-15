use crate::analysis::{format_recent_commits, group_files_by_type};
use crate::client::Result;

pub fn get_commit_system_prompt() -> String {
  "You are an expert at generating git commit messages following conventional commit standards. Your response should only contain the commit message, nothing else.".to_string()
}

pub async fn get_commit_user_prompt(
  branch_name: String,
  scopes: String,
  is_nx_repo: bool,
  diff: String,
  modified_files: Vec<String>,
  recent_commits: Vec<String>,
  change_analysis: String,
) -> Result<String> {
  // Group files by type
  let grouped_files = group_files_by_type(modified_files.clone());

  // Format recent commits
  let recent_commits_str = format_recent_commits(recent_commits);

  Ok(format!("# Git Commit Message Generation Prompt

You are an expert at writing clear, concise, and meaningful git commit messages following conventional commit patterns.

## Requirements

### Conventional Commit Format
Follow the pattern: `<type>[optional scope]: <description>`

### Types
- **feat**: A new feature
- **fix**: A bug fix
- **docs**: Documentation only changes
- **style**: Changes that do not affect the meaning of the code (white-space, formatting, missing semi-colons, etc)
- **refactor**: A code change that neither fixes a bug nor adds a feature
- **perf**: A code change that improves performance
- **test**: Adding missing tests or correcting existing tests
- **chore**: Changes to the build process or auxiliary tools and libraries
- **ci**: Changes to CI configuration files and scripts
- **build**: Changes that affect the build system or external dependencies

### Guidelines
- **Keep it short**: Limit the subject line to 72 characters or less
- **Use imperative mood**: \"Add feature\" not \"Added feature\" or \"Adding feature\"
- **No period**: Don't end the subject line with a period
- **Capitalize**: Start with a capital letter
- **Be specific**: Describe what the commit does, not what was wrong

### Scope Examples

### Scope Guidelines

- If a comma-separated list of scopes is provided in the data, use ONLY scopes from this list
- For Nx repositories (when is_nx_repo is true), the scopes list will contain app and lib directory names
- Only use a scope if it clearly matches the changes being made
- If no scope from the provided list is suitable, omit the scope entirely
- Do not invent scopes that aren't in the provided list
- For Nx repositories, prefer using the app or lib name that contains the changed files as the scope
- If the scopes list is empty and it's not an Nx repo, you may derive a scope from the directory names in the diff

Examples with scope:
- `feat(helix): add typescript lsp support`
- `fix(nvim): resolve plugin loading issue`
- `docs(fish): update function documentation`

Examples without scope:
- `feat: add new configuration option`
- `fix: resolve cross-platform compatibility issue`
- `refactor: simplify error handling logic`

### Multi-file Changes
- Focus on the primary purpose of the change
- Use the most appropriate type for the overall change
- Consider breaking large changes into smaller, focused commits

### Message Structure
When multiple changes are present:
- **Subject**: Describe the major/primary change (max 72 chars)
- **Body**: Use bullet points with `-` for additional changes
- **Footer**: Include breaking changes and issue references

### Body Format
```
- Add secondary feature or fix
- Update documentation for new API
- Refactor helper functions for better performance
```

### Footer Format
```
BREAKING CHANGE: API endpoint /users now requires authentication

Closes #123
Fixes #456
Resolves #789
```

### Breaking Changes
- Always include `BREAKING CHANGE:` in footer when applicable
- Describe what changed and migration path if needed
- Use when changes break backward compatibility

### Issue References
- Use `Closes #123` for features that close issues
- Use `Fixes #456` for bug fixes that resolve issues
- Use `Resolves #789` for general issue resolution
- Multiple references are allowed

Generate commit messages that clearly communicate the intent and impact of the staged changes.

Analyze the branch name, diff and scopes attached, to generate a conventional commit message.

```md
Branch name: {}
Scopes: {}
Is Nx Repository: {}

Diff of staged changes:
{}

Modified files:
{}

Recent commits:
{}

Change analysis:
{}
```

ALWAYS RETURN COMMIT MESSAGE as STANDARD OUTPUT LIKE FOLLOWING AND NO EXPLANATION, NO INTRODUCTION, NO SUMMARY,
JUST commit message, like following.

<message-here>
", branch_name, scopes, is_nx_repo, diff, grouped_files, recent_commits_str, change_analysis))
}
