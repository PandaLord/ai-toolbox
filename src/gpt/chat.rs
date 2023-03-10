use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct ChatFormat {
    role: String,
    content: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GPTPayload {
    model: String,
    message: Vec<ChatFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    logprobs: Option<u32>,
    
}