use crate::anthropic::AnthropicClient;
use crate::openai::OpenAIClient;
use std::env;
use std::env::VarError;

#[derive(Debug, Clone)]
pub struct UsageInfo {
  pub input_tokens: u32,
  pub output_tokens: u32,
  pub total_tokens: u32,
}

#[derive(Debug)]
pub struct GenerateResponseResult {
  pub message: String,
  pub usage: UsageInfo,
}

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

pub trait AIClient: Send + Sync {
  fn set_model(&mut self, model: String);
  fn set_max_tokens(&mut self, max_tokens: u32);
  fn set_temperature(&mut self, temperature: f32);
  fn generate_response(
    &self,
    system_prompt: String,
    user_prompt: String,
  ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<GenerateResponseResult>> + Send + '_>>;
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

pub struct ClientBuilder {
  model: String,
  max_tokens: Option<u32>,
  temperature: Option<f32>,
}

impl ClientBuilder {
  pub fn new(model: &str) -> Self {
    Self {
      model: model.to_string(),
      max_tokens: None,
      temperature: None,
    }
  }

  pub fn max_tokens(mut self, max_tokens: u32) -> Self {
    self.max_tokens = Some(max_tokens);
    self
  }

  pub fn temperature(mut self, temperature: f32) -> Self {
    self.temperature = Some(temperature);
    self
  }

  pub fn build(self) -> std::result::Result<Box<dyn AIClient>, CreateClientError> {
    let (provider, model_name) = parse_model(&self.model)?;
    let api_key = get_provider_key(&provider)?;
    
    let max_tokens = self.max_tokens.unwrap_or(500);
    let temperature = self.temperature.unwrap_or(0.2);

    let mut client: Box<dyn AIClient> = match provider {
      ModelProvider::OpenAI => {
        Box::new(OpenAIClient::new(api_key))
      }
      ModelProvider::Anthropic => {
        Box::new(AnthropicClient::new(api_key))
      }
    };

    client.set_model(model_name);
    client.set_max_tokens(max_tokens);
    client.set_temperature(temperature);

    Ok(client)
  }
}

pub fn create_client(
  model: &str,
  max_tokens: u32,
) -> std::result::Result<Box<dyn AIClient>, CreateClientError> {
  ClientBuilder::new(model)
    .max_tokens(max_tokens)
    .build()
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
