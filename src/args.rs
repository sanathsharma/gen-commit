use clap::{Arg, ArgMatches, Command};
use std::env;

pub fn get_matches() -> ArgMatches {
  Command::new("gen-commit")
    .version(env!("CARGO_PKG_VERSION"))
    .about("Generate commit messages using AI models from Anthropic and OpenAI")
    .arg(
      Arg::new("dry-run")
        .short('n')
        .long("dry-run")
        .help("Show the generated commit message without committing")
        .default_value("false")
        .action(clap::ArgAction::SetTrue),
    )
    .arg(
      Arg::new("model")
        .short('m')
        .long("model")
        .help("Specify the model to use in format 'provider:model' (e.g., anthropic:claude-sonnet-4-20250514 or openai:gpt-4)")
        .env("GC_DEFAULT_MODEL")
        .default_value("anthropic:claude-sonnet-4-20250514")
        .value_name("MODEL"),
    )
    .arg(
      Arg::new("max-tokens")
        .short('t')
        .long("max-tokens")
        .help("Maximum number of tokens in the generated response")
        .value_name("COUNT")
        .value_parser(clap::value_parser!(u32))
        .default_value("500"),
    )
    .arg(
      Arg::new("ignore")
        .short('i')
        .long("ignore")
        .help("Comma-separated list of files or directories to ignore in the diff")
        .value_name("FILES") // This appears in help text to describe the expected value format
        .num_args(1)
        .env("GC_IGNORE_LIST")
        .default_value("package-lock.json,Cargo.lock,bun.lock,pnpm-lock.yaml"),
    )
    .get_matches()
}
