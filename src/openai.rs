use crate::client::{AIClient, ClientError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct OpenAIRequest {
  model: String,
  max_output_tokens: u32,
  input: String,
  stream: bool,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
  output: Vec<ContentBlock>,
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
  content: Vec<OutputText>,
}

#[derive(Debug, Deserialize)]
struct OutputText {
  text: String,
}

pub struct OpenAIClient {
  client: Client,
  api_key: String,
  model: String,
  max_tokens: u32,
}

impl OpenAIClient {
  pub fn new(api_key: String) -> Self {
    Self {
      client: Client::new(),
      api_key,
      model: "gpt-4.1".to_string(),
      max_tokens: 500,
    }
  }
}

impl AIClient for OpenAIClient {
  fn with_model(&mut self, model: String) -> &mut Self {
    self.model = model;
    self
  }

  fn with_max_tokens(&mut self, max_tokens: u32) -> &mut Self {
    self.max_tokens = max_tokens;
    self
  }

  async fn generate_commit_message_sync<T: Into<String>>(&self, user_prompt: T) -> Result<String> {
    let request = OpenAIRequest {
      model: self.model.clone(),
      max_output_tokens: self.max_tokens,
      input: user_prompt.into(),
      stream: false,
    };

    let response = self
      .client
      .post("https://api.openai.com/v1/responses")
      .header("Content-Type", "application/json")
      .header("Authorization", format!("Bearer {}", &self.api_key))
      .json(&request)
      .send()
      .await
      .map_err(|_| ClientError::FailedToSend)?;

    if !response.status().is_success() {
      let error_text = response.text().await.unwrap_or_default();
      return Err(ClientError::RequestFailed(error_text));
    }

    let api_response: OpenAIResponse = response
      .json()
      .await
      .map_err(|_| ClientError::FailedToParseResponse)?;

    Ok(
      api_response
        .output
        .first()
        .map(|block| block.content.first())
        .flatten()
        .map(|block| block.text.trim().to_string())
        .unwrap_or_default(),
    )
  }
}
