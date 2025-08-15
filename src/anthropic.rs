use crate::client::{AIClient, ClientError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct AnthropicRequest {
  model: String,
  max_tokens: u32,
  messages: Vec<Message>,
  stream: bool,
  temperature: f32,
	system: Vec<SystemMessage>,
}

#[derive(Debug, Serialize)]
struct Message {
  role: String,
  content: String,
}

#[derive(Debug, Serialize)]
struct SystemMessage {
  r#type: String,
  text: String,
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
  temperature: f32,
}

impl AnthropicClient {
  pub fn new(api_key: String) -> Self {
    Self {
      client: Client::new(),
      api_key,
      model: "claude-3-7-sonnet-20250219".to_string(),
      max_tokens: 500,
      temperature: 0.7,
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

  fn with_temperature(&mut self, temperature: f32) -> &mut Self {
    self.temperature = temperature;
    self
  }

  async fn generate_response<T: Into<String>>(
    &self,
    system_prompt: T,
    user_prompt: T,
  ) -> Result<String> {
    // Create system and user messages
    let system_message = SystemMessage {
      r#type: "text".to_string(),
      text: system_prompt.into(),
    };

    let user_message = Message {
      role: "user".to_string(),
      content: user_prompt.into(),
    };

    let request = AnthropicRequest {
      model: self.model.clone(),
      max_tokens: self.max_tokens,
			system: vec![system_message],
      messages: vec![user_message],
      stream: false,
      temperature: self.temperature,
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
