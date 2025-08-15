use crate::anthropic::AnthropicClient;
use crate::openai::OpenAIClient;
use std::env;
use std::env::VarError;

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
  #[error("Failed to send request to API")]
  FailedToSend,
  #[error("API request failed: {0}")]
  RequestFailed(String),
  #[error("Failed to parse API response")]
  FailedToParseResponse,
}

pub type Result<T> = std::result::Result<T, ClientError>;

pub trait AIClient {
  fn with_model(&mut self, model: String) -> &mut Self;
  fn with_max_tokens(&mut self, max_tokens: u32) -> &mut Self;
  fn with_temperature(&mut self, temperature: f32) -> &mut Self;
  async fn generate_response<T: Into<String>>(
    &self,
    system_prompt: T,
    user_prompt: T,
  ) -> Result<String>;
}

pub enum ModelProvider {
  OpenAI,
  Anthropic,
}

#[derive(thiserror::Error, Debug)]
pub enum ParseModelError {
  #[error("Invalid model format")]
  InvalidModelFormat,
}

#[derive(thiserror::Error, Debug)]
pub enum CreateClientError {
  #[error("Failed to parse model")]
  ParseError(#[from] ParseModelError),
  #[error(transparent)]
  APIKeyError(#[from] VarError),
}

pub enum Client {
  OpenAI(OpenAIClient),
  Anthropic(AnthropicClient),
}

pub fn create_client(
  model: &str,
  max_tokens: u32,
) -> std::result::Result<Client, CreateClientError> {
  let (provider, model) = parse_model(model)?;
  let api_key = get_provider_key(&provider)?;

  let client = match provider {
    ModelProvider::OpenAI => {
      let mut openai_client = OpenAIClient::new(api_key);
      openai_client
        .with_model(model)
        .with_max_tokens(max_tokens)
        .with_temperature(0.2);

      Client::OpenAI(openai_client)
    }
    ModelProvider::Anthropic => {
      let mut anthropic_client = AnthropicClient::new(api_key);
      anthropic_client
        .with_model(model)
        .with_max_tokens(max_tokens)
        .with_temperature(0.2);

      Client::Anthropic(anthropic_client)
    }
  };

  Ok(client)
}

fn parse_model(model: &str) -> std::result::Result<(ModelProvider, String), ParseModelError> {
  let mut parts = model.split(':');

  let provider_str = parts.next();
  let model_name = parts.next();

  // Check if there are more parts (which would be invalid)
  if parts.next().is_some() {
    return Err(ParseModelError::InvalidModelFormat);
  }

  match (provider_str, model_name) {
    (Some("openai"), Some(m)) => Ok((ModelProvider::OpenAI, m.to_string())),
    (Some("anthropic"), Some(m)) => Ok((ModelProvider::Anthropic, m.to_string())),
    _ => Err(ParseModelError::InvalidModelFormat),
  }
}

fn get_provider_key(provider: &ModelProvider) -> std::result::Result<String, VarError> {
  let key = match provider {
    ModelProvider::OpenAI => "OPENAI_API_KEY",
    ModelProvider::Anthropic => "ANTHROPIC_API_KEY",
  };

  env::var(key)
}
