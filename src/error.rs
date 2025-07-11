use crate::client::{ClientError, CreateClientError};
use crate::git;
use std::env;

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("ANTHROPIC_API_KEY environment variable not found")]
  CouldNotFindAnthropicKey(#[from] env::VarError),
  #[error(transparent)]
  GitError(#[from] git::GitError),
  #[error(transparent)]
  IoError(#[from] std::io::Error),
  #[error(transparent)]
  ClientError(#[from] ClientError),
  #[error(transparent)]
  CreateClientError(#[from] CreateClientError),
}

pub type Result<T> = std::result::Result<T, Error>;
