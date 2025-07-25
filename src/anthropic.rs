use crate::client::{AIClient, ClientError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct AnthropicRequest {
  model: String,
  max_tokens: u32,
  messages: Vec<Message>,
  stream: bool,
}

#[derive(Debug, Serialize)]
struct Message {
  role: String,
  content: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
  content: Vec<ContentBlock>,
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
  text: String,
}

pub struct AnthropicClient {
  client: Client,
  api_key: String,
  model: String,
  max_tokens: u32,
}

impl AnthropicClient {
  pub fn new(api_key: String) -> Self {
    Self {
      client: Client::new(),
      api_key,
      model: "claude-3-7-sonnet-20250219".to_string(),
      max_tokens: 500,
    }
  }
}

impl AIClient for AnthropicClient {
  fn with_model(&mut self, model: String) -> &mut Self {
    self.model = model;
    self
  }

  fn with_max_tokens(&mut self, max_tokens: u32) -> &mut Self {
    self.max_tokens = max_tokens;
    self
  }

  async fn generate_commit_message_sync<T: Into<String>>(&self, user_prompt: T) -> Result<String> {
    let request = AnthropicRequest {
      model: self.model.clone(),
      max_tokens: self.max_tokens,
      messages: vec![Message {
        role: "user".to_string(),
        content: user_prompt.into(),
      }],
      stream: false,
    };

    let response = self
      .client
      .post("https://api.anthropic.com/v1/messages")
      .header("Content-Type", "application/json")
      .header("x-api-key", &self.api_key)
      .header("anthropic-version", "2023-06-01")
      .json(&request)
      .send()
      .await
      .map_err(|_| ClientError::FailedToSend)?;

    if !response.status().is_success() {
      let error_text = response.text().await.unwrap_or_default();
      return Err(ClientError::RequestFailed(error_text));
    }

    let api_response: AnthropicResponse = response
      .json()
      .await
      .map_err(|_| ClientError::FailedToParseResponse)?;

    Ok(
      api_response
        .content
        .first()
        .map(|block| block.text.trim().to_string())
        .unwrap_or_default(),
    )
  }
}
