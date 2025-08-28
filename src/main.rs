#![allow(dead_code)]

use crate::git::is_git_repo;
use crate::logs::{LogLevel, Logger};

mod analysis;
mod anthropic;
mod args;
mod client;
mod error;
mod file;
mod git;
mod logs;
mod openai;
mod prompt;

#[tokio::main]
async fn main() -> error::Result<()> {
  if !is_git_repo().await {
    eprintln!("not a git repository");
    std::process::exit(1);
  };

  let matches = args::get_matches();

  let log_level = if matches.get_flag("verbose") {
    LogLevel::Verbose
  } else {
    LogLevel::None
  };
  let logger = Logger::new(log_level);

  logger.log_step("Initializing gen-commit");

  logger.log_step("Getting git root directory");
  let root_dir = git::get_git_root().await?;
  logger.log_output(&format!("Git root: {}", root_dir));

  let mut ignore_list: Vec<String> = matches
    .get_one::<String>("ignore")
    .map(|s| {
      s.split(',')
        .filter(|item| !item.trim().is_empty())
        .map(|item| format!(":!{}", item.trim()))
        .collect()
    })
    .unwrap_or_default();

  logger.log_step("Getting current branch name");
  let branch_name = git::get_branch_name().await?;
  logger.log_output(&format!("Branch: {}", branch_name));
  logger.log_step("Reading scopes file");
  let scopes = file::read_file(format!("{root_dir}/scopes.txt"))
    .await
    .unwrap_or_default();
  logger.log_output(&format!("Scopes found: {}", !scopes.is_empty()));
  logger.log_step("Checking for Nx repository");
  let is_nx_repo = file::file_exists(format!("{root_dir}/nx.json"));
  logger.log_output(&format!("Nx repo: {}", is_nx_repo));

  logger.log_step("Getting staged diff");
  let diff = git::get_staged_diff(&mut ignore_list).await?;
  logger.log_output(&format!("Diff length: {} characters", diff.len()));

  if diff.is_empty() {
    eprintln!("no changes detected");
    std::process::exit(1);
  }

  logger.log_step("Getting modified files");
  let modified_files = git::get_modified_files().await?;
  logger.log_output(&format!("Modified files count: {}", modified_files.len()));
  if !modified_files.is_empty() {
    logger.log_output("Modified files:");
    for file in &modified_files {
      logger.log_output(&format!("  - {}", file));
    }
  }

  logger.log_step("Getting recent commits");
  let recent_commits = git::get_recent_commits(5).await.unwrap_or_else(|_| {
    logger.log_output("Warning: Unable to get recent commits, continuing without them");
    Vec::new()
  });
  logger.log_output(&format!("Recent commits count: {}", recent_commits.len()));
  if !recent_commits.is_empty() {
    logger.log_output("Recent commits:");
    for commit in &recent_commits {
      logger.log_output(&format!("  - {}", commit));
    }
  }

  logger.log_step("Creating AI client");
  let client = client::create_client(
    matches.get_one::<String>("model").unwrap(),
    *matches.get_one::<u32>("max-tokens").unwrap(),
  )?;
  logger.log_output(&format!(
    "Model: {}",
    matches.get_one::<String>("model").unwrap()
  ));

  let (analysis_message, analysis_usage) = if matches.get_flag("no-analysis") {
    logger.log_step("Skipping AI analysis (--no-analysis flag enabled)");
    (String::new(), None)
  } else {
    logger.log_step("Analyzing changes with AI");
    let analysis_response = analysis::analyze_changes_with_ai(client.as_ref(), &diff).await?;
    logger.log_output(&format!(
      "Change analysis length: {} characters",
      analysis_response.message.len()
    ));
    logger.log_output(&format!(
      "Analysis usage - Input: {}, Output: {}, Total: {}",
      analysis_response.usage.input_tokens,
      analysis_response.usage.output_tokens,
      analysis_response.usage.total_tokens
    ));
    (analysis_response.message, Some(analysis_response.usage))
  };

  logger.log_step("Building user prompt");
  let user_prompt = prompt::get_commit_user_prompt(
    branch_name,
    scopes,
    is_nx_repo,
    diff,
    modified_files,
    recent_commits,
    analysis_message,
  )
  .await?;
  logger.log_output(&format!(
    "User prompt length: {} characters",
    user_prompt.len()
  ));

  logger.log_step("Generating commit message");
  let response = client
    .generate_response(prompt::get_commit_system_prompt(), user_prompt)
    .await?;

  let commit_message = response.message;

  println!("Generated commit message:\n");
  println!("{commit_message}");

  // Log individual usage in verbose mode using logger
  logger.log_output("--- Individual Usage ---");
  if let Some(usage) = &analysis_usage {
    logger.log_output("Analysis:");
    logger.log_output(&format!("  Input tokens: {}", usage.input_tokens));
    logger.log_output(&format!("  Output tokens: {}", usage.output_tokens));
    logger.log_output(&format!("  Total tokens: {}", usage.total_tokens));
  }

  logger.log_output("Commit Message Generation:");
  logger.log_output(&format!("  Input tokens: {}", response.usage.input_tokens));
  logger.log_output(&format!(
    "  Output tokens: {}",
    response.usage.output_tokens
  ));
  logger.log_output(&format!("  Total tokens: {}", response.usage.total_tokens));

  // Calculate total usage
  let (total_input, total_output, total_tokens) = if let Some(usage) = &analysis_usage {
    (
      usage.input_tokens + response.usage.input_tokens,
      usage.output_tokens + response.usage.output_tokens,
      usage.total_tokens + response.usage.total_tokens,
    )
  } else {
    (
      response.usage.input_tokens,
      response.usage.output_tokens,
      response.usage.total_tokens,
    )
  };

  println!("\n--- Total Usage ---");
  println!("  Input tokens: {}", total_input);
  println!("  Output tokens: {}", total_output);
  println!("  Total tokens: {}", total_tokens);

  if !matches.get_flag("dry-run") {
    println!("\nCommit with this message? (y/N)");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim().to_lowercase() == "y" {
      logger.log_step("Committing changes");
      git::commit(&commit_message).await?;
      logger.log_output("Commit successful");
      println!("Successfully committed!");
    } else {
      println!("Commit cancelled.");
    }
  }

  Ok(())
}
