#![allow(dead_code)]

use crate::client::UsageInfo;
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

struct AppContext {
  branch_name: String,
  scopes: String,
  is_nx_repo: bool,
  diff: String,
  modified_files: Vec<String>,
  recent_commits: Vec<String>,
}

async fn initialize_app() -> error::Result<(Logger, clap::ArgMatches)> {
  if !is_git_repo().await {
    eprintln!("not a git repository");
    std::process::exit(1);
  }

  let matches = args::get_matches();
  let log_level = if matches.get_flag("verbose") {
    LogLevel::Verbose
  } else {
    LogLevel::None
  };
  let logger = Logger::new(log_level);
  logger.log_step("Initializing gen-commit");

  Ok((logger, matches))
}

async fn gather_git_context(logger: &Logger, ignore_list: &mut Vec<String>) -> error::Result<AppContext> {
  logger.log_step("Getting git root directory");
  let root_dir = git::get_git_root().await?;
  logger.log_output(&format!("Git root: {}", root_dir));

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
  let diff = git::get_staged_diff(ignore_list).await?;
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

  Ok(AppContext {
    branch_name,
    scopes,
    is_nx_repo,
    diff,
    modified_files,
    recent_commits,
  })
}

async fn process_with_ai(
  logger: &Logger,
  matches: &clap::ArgMatches,
  context: &AppContext,
) -> error::Result<(String, Option<UsageInfo>, UsageInfo)> {
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
    let analysis_response = analysis::analyze_changes_with_ai(client.as_ref(), &context.diff).await?;
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
    context.branch_name.clone(),
    context.scopes.clone(),
    context.is_nx_repo,
    context.diff.clone(),
    context.modified_files.clone(),
    context.recent_commits.clone(),
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

  Ok((response.message, analysis_usage, response.usage))
}

fn report_usage(logger: &Logger, analysis_usage: &Option<UsageInfo>, generation_usage: &UsageInfo) {
  logger.log_output("--- Individual Usage ---");
  if let Some(usage) = analysis_usage {
    logger.log_output("Analysis:");
    logger.log_output(&format!("  Input tokens: {}", usage.input_tokens));
    logger.log_output(&format!("  Output tokens: {}", usage.output_tokens));
    logger.log_output(&format!("  Total tokens: {}", usage.total_tokens));
  }

  logger.log_output("Commit Message Generation:");
  logger.log_output(&format!("  Input tokens: {}", generation_usage.input_tokens));
  logger.log_output(&format!("  Output tokens: {}", generation_usage.output_tokens));
  logger.log_output(&format!("  Total tokens: {}", generation_usage.total_tokens));

  let (total_input, total_output, total_tokens) = if let Some(usage) = analysis_usage {
    (
      usage.input_tokens + generation_usage.input_tokens,
      usage.output_tokens + generation_usage.output_tokens,
      usage.total_tokens + generation_usage.total_tokens,
    )
  } else {
    (
      generation_usage.input_tokens,
      generation_usage.output_tokens,
      generation_usage.total_tokens,
    )
  };

  println!("\n--- Total Usage ---");
  println!("  Input tokens: {}", total_input);
  println!("  Output tokens: {}", total_output);
  println!("  Total tokens: {}", total_tokens);
}

async fn handle_commit_confirmation(
  logger: &Logger,
  commit_message: &str,
  is_dry_run: bool,
) -> error::Result<()> {
  if !is_dry_run {
    println!("\nCommit with this message? (y/N)");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim().to_lowercase() == "y" {
      logger.log_step("Committing changes");
      git::commit(commit_message).await?;
      logger.log_output("Commit successful");
      println!("Successfully committed!");
    } else {
      println!("Commit cancelled.");
    }
  }
  Ok(())
}

#[tokio::main]
async fn main() -> error::Result<()> {
  let (logger, matches) = initialize_app().await?;

  let mut ignore_list: Vec<String> = matches
    .get_one::<String>("ignore")
    .map(|s| {
      s.split(',')
        .filter(|item| !item.trim().is_empty())
        .map(|item| format!(":!{}", item.trim()))
        .collect()
    })
    .unwrap_or_default();

  let context = gather_git_context(&logger, &mut ignore_list).await?;

  let (commit_message, analysis_usage, generation_usage) = process_with_ai(
    &logger,
    &matches,
    &context,
  ).await?;

  println!("Generated commit message:\n");
  println!("{commit_message}");

  report_usage(&logger, &analysis_usage, &generation_usage);

  handle_commit_confirmation(&logger, &commit_message, matches.get_flag("dry-run")).await?;

  Ok(())
}
