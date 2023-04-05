use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Debug, Deserialize)]
pub enum Model {
    // gpt 3.5 turbo model
    #[default]
    #[serde(rename = "gpt-3.5-turbo")]
    Gpt35Turbo,

    // GPT turbo 0301 model
    #[serde(rename = "gpt-3.5-turbo-0301")]
    Gpt35Turbo0301,

    // Text Embedding Ada 002
    #[serde(rename = "text-embedding-ada-002")]
    Ada002,
    // Text Embedding Ada 002 v2
    #[serde(rename = "text-embedding-ada-002-v2")]
    Ada002V2,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    /// Index of the message
    pub index: u32,

    /// text of chat completion
    pub message: Message,

    /// finish reason for the chat completion
    pub finish_reason: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Message {
    /// role for the message
    pub role: String,

    /// message content
    pub content: String,
}

#[derive(Serialize, Default)]
pub struct ChatPayload {
    /// ID of the model to use. Currently, only `gpt-3.5-turbo` and `gpt-3.5-turbo-0301` are supported.
    pub model: Model,

    /// Messages to generate chat completions for, in the chat format.
    pub messages: Vec<Message>,

    /// What sampling temperature to use, between 0 and 2. Higher values like 0.8 will make the output more random, while lower values like 0.2 will make it more focused and deterministic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,

    /// Alternative to sampling with temperature, called nucleus sampling, where the model considers the results of the tokens with top_p probability mass.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,

    /// How many chat completion choices to generate for each input message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<i32>,

    /// Up to 4 sequences where the API will stop generating further tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,

    /// Maximum number of tokens to generate for each chat completion choice.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i32>,

    /// A number between -2.0 and 2.0. Positive values penalize new tokens based on whether they appear in the text so far, increasing the model's likelihood to talk about new topics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f64>,

    /// A number between -2.0 and 2.0. Positive values penalize new tokens based on their existing frequency in the text so far, decreasing the model's likelihood to repeat the same line verbatim.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f64>,

    /// A unique identifier representing your end-user, which can help OpenAI to monitor and detect abuse.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}
#[derive(Debug, Deserialize)]
/// usage information for the OpenAI API.
pub struct Usage {
    /// how many tokens were used for the prompt.
    pub prompt_tokens: i32,

    /// how many tokens were used for the chat completion.
    
    #[serde(skip_deserializing)]
    pub completion_tokens: i32,

    /// how many tokens were used for the entire request.
    pub total_tokens: i32,
}

#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    /// ID of the request.
    pub id: String,
    pub object: String,

    /// when the request was created.
    pub created: i64,

    /// list of chat completion choices.
    pub choices: Vec<Choice>,

    /// usage information for the request.
    pub usage: Usage,
}

/*
   Model Relevant Data Structure
*/

// Model Response
#[derive(Debug, Deserialize)]
pub struct ModelResponse {
    pub data: Vec<ModelData>,
    pub object: String,
}

// Model data object
#[derive(Debug, Deserialize)]
pub struct ModelData {
    pub id: String,
    pub object: String,
    pub owned_by: String,
    pub permission: Vec<PermissionData>,
}

#[derive(Debug, Deserialize)]
pub struct PermissionData {
    pub allow_create_engine: bool,
    pub allow_fine_tuning: bool,
    pub allow_logprobs: bool,
    pub allow_sampling: bool,
    pub allow_search_indices: bool,
    pub allow_view: bool,
}

#[derive(Debug, Serialize, Default)]
pub struct EmbeddingPayload {
    pub model: Model,
    pub input: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EmbeddingResponse {
    pub object: String,
    pub data: Vec<EmbeddingData>,
    pub model: Model,
    pub usage: Usage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingData {
    pub object: String,
    pub embedding: Vec<f32>,
    pub index: u32,
}

impl Display for ChatResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut content = "".to_string();
        for data in &self.choices {
            content += data.message.content.as_str();
        }
        write!(f, "{}", content)
    }
}
