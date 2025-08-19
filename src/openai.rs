use crate::client::{AIClient, ClientError, GenerateResponseResult, Result, UsageInfo};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct OpenAIMessage {
  role: String,
  content: String,
}

#[derive(Debug, Serialize)]
struct OpenAIRequest {
  model: String,
  input: Vec<OpenAIMessage>,
  max_tokens: u32,
  temperature: f32,
  stream: bool,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
  output: Vec<ContentBlock>,
  usage: OpenAIUsage,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
  input_tokens: u32,
  output_tokens: u32,
  total_tokens: u32,
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
  temperature: f32,
}

impl OpenAIClient {
  pub fn new(api_key: String) -> Self {
    Self {
      client: Client::new(),
      api_key,
      model: "gpt-4.1".to_string(),
      max_tokens: 500,
      temperature: 0.7,
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

  fn with_temperature(&mut self, temperature: f32) -> &mut Self {
    self.temperature = temperature;
    self
  }

  async fn generate_response<T: Into<String>>(
    &self,
    system_prompt: T,
    user_prompt: T,
  ) -> Result<GenerateResponseResult> {
    let system_message = OpenAIMessage {
      role: "system".to_string(),
      content: system_prompt.into(),
    };

    let user_message = OpenAIMessage {
      role: "user".to_string(),
      content: user_prompt.into(),
    };

    let request = OpenAIRequest {
      model: self.model.clone(),
      input: vec![system_message, user_message],
      max_tokens: self.max_tokens,
      temperature: self.temperature,
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

    let message = api_response
      .output
      .first()
      .and_then(|block| block.content.first())
      .map(|block| block.text.trim().to_string())
      .unwrap_or_default();

    let usage = UsageInfo {
      input_tokens: api_response.usage.input_tokens,
      output_tokens: api_response.usage.output_tokens,
      total_tokens: api_response.usage.total_tokens,
    };

    Ok(GenerateResponseResult { message, usage })
  }
}
