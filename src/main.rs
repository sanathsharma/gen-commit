use crate::anthropic::AnthropicClient;
use crate::git::is_git_repo;
use std::env;

mod anthropic;
mod args;
mod error;
mod file;
mod git;
mod prompt;

#[tokio::main]
async fn main() -> error::Result<()> {
  if !is_git_repo().await {
    eprintln!("not a git repository");
    std::process::exit(1);
  };

  let matches = args::get_matches();
  let anthropic_key = env::var("ANTHROPIC_API_KEY")?;
  let root_dir = git::get_git_root().await?;

  let mut ignore_list: Vec<String> = matches
    .get_one::<String>("ignore")
    .map(|s| {
      s.split(',')
        .map(|item| format!(":!{}", item.trim()))
        .collect()
    })
    .unwrap_or_default();

  let branch_name = git::get_branch_name().await?;
  let scopes = file::read_file(format!("{}/scopes.txt", root_dir))
    .await
    .unwrap_or_default();
  let is_nx_repo = file::file_exists(format!("{}/nx.json", root_dir));
  let diff = git::get_staged_diff(&mut ignore_list).await?;

  let user_prompt = prompt::get_user_prompt(branch_name, scopes, is_nx_repo, diff);

  let anthropic_client = AnthropicClient::new(anthropic_key)
    .with_model(matches.get_one::<String>("model").unwrap().clone())
    .with_max_tokens(*matches.get_one::<u32>("max-tokens").unwrap());

  let commit_message = anthropic_client
    .generate_commit_message_sync(user_prompt)
    .await?;

  println!("Generated commit message:\n");
  println!("{}", commit_message);

  if !matches.get_flag("dry-run") {
    println!("\nCommit with this message? (y/N)");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim().to_lowercase() == "y" {
      git::commit(&commit_message).await?;
      println!("Successfully committed!");
    } else {
      println!("Commit cancelled.");
    }
  }

  Ok(())
}
