use tokio::process::Command;

#[derive(Debug, thiserror::Error, Clone)]
pub enum GitError {
  #[error("No changes staged")]
  NoStagedChanges,
  #[error("Failed to execute {0}")]
  FailedToExecuteCmd(String),
}

type Result<T> = std::result::Result<T, GitError>;

pub async fn is_git_repo() -> bool {
  let output = Command::new("git")
    .args(&["rev-parse", "--git-dir"])
    .current_dir(".")
    .output()
    .await;

  let output = match output {
    Ok(output) => output,
    Err(_) => return false,
  };

  output.status.success()
}

pub async fn get_staged_diff(ignore_list: &mut Vec<String>) -> Result<String> {
  let mut args: Vec<String> = Vec::from(["diff".to_string(), "--staged".to_string()]);
  if !ignore_list.is_empty() {
    args.push("--".to_string());
    args.append(ignore_list);
  }

  let output = Command::new("git")
    .args(&args)
    .current_dir(".")
    .output()
    .await
    .map_err(|_| GitError::FailedToExecuteCmd(String::from("git diff --staged")))?;

  if !output.status.success() {
    return Err(GitError::NoStagedChanges);
  }

  Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub async fn get_modified_files() -> Result<Vec<String>> {
  let output = Command::new("git")
    .args(&["diff", "--name-only", "--staged"])
    .current_dir(".")
    .output()
    .await
    .map_err(|_| GitError::FailedToExecuteCmd(String::from("git diff --name-only --staged")))?;

  if !output.status.success() {
    return Err(GitError::NoStagedChanges);
  }

  let files = String::from_utf8_lossy(&output.stdout)
    .lines()
    .map(|s| s.to_string())
    .collect();

  Ok(files)
}

pub async fn get_recent_commits(count: usize) -> Result<Vec<String>> {
  let err = || GitError::FailedToExecuteCmd(String::from("git log"));
  let output = Command::new("git")
    .args(&["log", "--oneline", "-n", &count.to_string()])
    .current_dir(".")
    .output()
    .await
    .map_err(|_| err())?;

  if !output.status.success() {
    return Err(err());
  }

  let log = String::from_utf8_lossy(&output.stdout).to_string();
  let commits = log
    .lines()
    .map(|line| {
      // Skip the commit hash and just get the message
      line.splitn(2, ' ').nth(1).unwrap_or("").to_string()
    })
    .collect();

  Ok(commits)
}

pub async fn commit(message: &str) -> Result<()> {
  let mut child = Command::new("git")
    .args(&["commit", "-m", message, "-e"])
    .current_dir(".")
    .spawn()
    .map_err(|_| GitError::FailedToExecuteCmd(String::from("git commit")))?;

  let _ = child.wait().await;

  Ok(())
}

pub async fn get_branch_name() -> Result<String> {
  let err = || GitError::FailedToExecuteCmd(String::from("git branch --show-current"));
  let output = Command::new("git")
    .args(&["branch", "--show-current"])
    .current_dir(".")
    .output()
    .await
    .map_err(|_| err())?;

  if !output.status.success() {
    return Err(err());
  }

  Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub async fn get_git_root() -> Result<String> {
  let err = || GitError::FailedToExecuteCmd(String::from("git rev-parse --show-toplevel"));
  let output = Command::new("git")
    .args(&["rev-parse", "--show-toplevel"])
    .current_dir(".")
    .output()
    .await
    .map_err(|_| err())?;

  if !output.status.success() {
    return Err(err());
  }

  Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
