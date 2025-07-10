use clap::{Arg, ArgMatches, Command};

pub fn get_matches() -> ArgMatches {
  Command::new("gen-commit")
    .version(env!("CARGO_PKG_VERSION"))
    .about("Generate commit messages using Anthropic API")
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
        .help("Specify the Anthropic model to use (default: claude-sonnet-4-20250514)")
        .default_value("claude-sonnet-4-20250514"),
    )
    .arg(
      Arg::new("max-tokens")
        .short('t')
        .long("max-tokens")
        .help("Maximum number of tokens in the generated response (default: 500)")
        .value_parser(clap::value_parser!(u32))
        .default_value("500"),
    )
    .get_matches()
}
