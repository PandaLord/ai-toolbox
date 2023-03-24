

use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::fmt::Display;


/// 要么是服务器问题,要么是请求有问题,Error 处理这两种情况
pub type ApiResult<T> = Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    ///show message as error
    #[error("OpenAI Error: {0}")]
    ApiError(GPTErrorResponse),

    #[error("Reqwest Error: {0}")]
    RequestError(#[from] reqwest::Error),
}

#[derive(Error, Debug, Serialize, Deserialize)]
pub struct GPTErrorResponse {
    pub error: GPTError,
}
impl Display for GPTErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "API Error Response: {}", self.error)
    }
}
#[derive(Error, Debug, Serialize, Deserialize)]
pub enum GPTError {
    ChatError(ChatError),
    EditError(EditError),
}

impl Display for GPTError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ChatError(e) => {
                write!(f, "Chat API Error, {}: {}", e.error_type, e.message)
            },
            Self::EditError(_) => {
                write!(f, "OpenAI Edit API Error")
            },
            (other) => {
                write!(f, "Unknown Error: {:?}", other)
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ChatError {
    /// error message.
    pub message: String,

    #[serde(rename = "type")]
    /// error type.
    pub error_type: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EditError {}

// #[derive(Error, Debug)]
// pub enum GPTError {
//     #[error("get request build error")]
//     ApiError(GPTErrorResponse),
//     #[error("post request build error")]
//     RequestError(String),
//     #[error("unknown request error")]
//     Unknown,
// }
