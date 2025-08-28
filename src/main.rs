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

async fn gather_git_context(
  logger: &Logger,
  ignore_list: &mut Vec<String>,
) -> error::Result<AppContext> {
  let root_dir = logger
    .exec_result_with_output(
      "Getting git root directory",
      || git::get_git_root(),
      |root| format!("Git root: {}", root),
    )
    .await?;

  let branch_name = logger
    .exec_result_with_output(
      "Getting current branch name",
      || git::get_branch_name(),
      |branch| format!("Branch: {}", branch),
    )
    .await?;

  let scopes = logger
    .exec_with_output(
      "Reading scopes file",
      || async {
        file::read_file(format!("{root_dir}/scopes.txt"))
          .await
          .unwrap_or_default()
      },
      |scopes| format!("Scopes found: {}", !scopes.is_empty()),
    )
    .await;

  let is_nx_repo = logger
    .exec_with_output(
      "Checking for Nx repository",
      || async { file::file_exists(format!("{root_dir}/nx.json")) },
      |nx| format!("Nx repo: {}", nx),
    )
    .await;

  let diff = logger
    .exec_result_with_output(
      "Getting staged diff",
      || git::get_staged_diff(ignore_list),
      |d| format!("Diff length: {} characters", d.len()),
    )
    .await?;

  if diff.is_empty() {
    eprintln!("no changes detected");
    std::process::exit(1);
  }

  let modified_files = logger
    .exec_result_with_output(
      "Getting modified files",
      || git::get_modified_files(),
      |files| {
        let mut output = format!("Modified files count: {}", files.len());
        if !files.is_empty() {
          output.push_str("\nModified files:");
          for file in files {
            output.push_str(&format!("\n  - {}", file));
          }
        }
        output
      },
    )
    .await?;

  let recent_commits = logger
    .exec_with_output(
      "Getting recent commits",
      || async {
        git::get_recent_commits(5).await.unwrap_or_else(|_| {
          println!("[OUTPUT] Warning: Unable to get recent commits, continuing without them");
          Vec::new()
        })
      },
      |commits| {
        let mut output = format!("Recent commits count: {}", commits.len());
        if !commits.is_empty() {
          output.push_str("\nRecent commits:");
          for commit in commits {
            output.push_str(&format!("\n  - {}", commit));
          }
        }
        output
      },
    )
    .await;

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
  let client = logger.exec_sync_result_with_output(
    "Creating AI client",
    || {
      client::create_client(
        matches.get_one::<String>("model").unwrap(),
        *matches.get_one::<u32>("max-tokens").unwrap(),
      )
    },
    |_| format!("Model: {}", matches.get_one::<String>("model").unwrap()),
  )?;

  let (analysis_message, analysis_usage) = if matches.get_flag("no-analysis") {
    logger.log_step("Skipping AI analysis (--no-analysis flag enabled)");
    (String::new(), None)
  } else {
    let analysis_response = logger.exec_result_with_output(
      "Analyzing changes with AI",
      || analysis::analyze_changes_with_ai(client.as_ref(), &context.diff),
      |resp| format!(
        "Change analysis length: {} characters\nAnalysis usage - Input: {}, Output: {}, Total: {}\nAnalysis content:\n{}",
        resp.message.len(),
        resp.usage.input_tokens,
        resp.usage.output_tokens,
        resp.usage.total_tokens,
        resp.message
      )
    ).await?;
    (analysis_response.message, Some(analysis_response.usage))
  };

  let user_prompt = logger
    .exec_result_with_output(
      "Building user prompt",
      || {
        prompt::get_commit_user_prompt(
          context.branch_name.clone(),
          context.scopes.clone(),
          context.is_nx_repo,
          context.diff.clone(),
          context.modified_files.clone(),
          context.recent_commits.clone(),
          analysis_message,
        )
      },
      |prompt| format!("User prompt length: {} characters", prompt.len()),
    )
    .await?;

  let response = logger
    .exec_result("Generating commit message", || {
      client.generate_response(prompt::get_commit_system_prompt(), user_prompt)
    })
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
  logger.log_output(&format!(
    "  Input tokens: {}",
    generation_usage.input_tokens
  ));
  logger.log_output(&format!(
    "  Output tokens: {}",
    generation_usage.output_tokens
  ));
  logger.log_output(&format!(
    "  Total tokens: {}",
    generation_usage.total_tokens
  ));

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
      logger
        .exec_result_with_output(
          "Committing changes",
          || git::commit(commit_message),
          |_| "Commit successful".to_string(),
        )
        .await?;
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

  let (commit_message, analysis_usage, generation_usage) =
    process_with_ai(&logger, &matches, &context).await?;

  println!("Generated commit message:\n");
  println!("{commit_message}");

  report_usage(&logger, &analysis_usage, &generation_usage);

  handle_commit_confirmation(&logger, &commit_message, matches.get_flag("dry-run")).await?;

  Ok(())
}
