use reqwest::{
    header::HeaderMap,
    header::{AUTHORIZATION, CONTENT_TYPE},
    Client,
};
use serde::Serialize;
use std::collections::HashMap;

use crate::{
    datamap::{
        ApiMethod, ApiPayload, ApiResponse, CompletionPayload, CompletionResponse,
        EmbeddingPayload, EmbeddingResponse, Response,
    },
    error::GPTError,
    openai::error::{Error, GPTErrorResponse},
};

use super::{
    datamap::{ChatPayload, ChatResponse, ModelResponse},
    error::ApiResult,
    token::Token,
};

// abstract Api struct to call different apis
#[derive(Debug, Clone)]
pub struct Api {
    organization_id: Option<String>,
    client: Client,
    api_dict: HashMap<String, String>,
}

impl Api {
    const BASE_URL: &'static str = "https://api.openai.com/v1";

    pub fn new(key: Token) -> Api {
        let mut header_map = HeaderMap::new();
        let api_json = r#"{
            "createChat": "/chat/completions",
            "createEdit": "/edits",
            "createImage": "/images/generations",
            "createImageEdit": "/images/edits",
            "createImageVariation": "/images/variations",
            "createEmbedding": "/embeddings",
            "createTranscription": "/audio/transcriptions",
            "createTranslation": "/audio/translations",
            "listModels": "/models",
            "createCompletion": "/completions"
        }"#;
        let api_dict: HashMap<String, String> =
            serde_json::from_str(api_json).expect("unable to transfer to hashmap");

        // header_map.insert(CONTENT_TYPE, "application/json".parse().unwrap());
        header_map.insert(CONTENT_TYPE, "application/json".parse().unwrap());
        header_map.insert(AUTHORIZATION, key.to_string().parse().unwrap());
        Api {
            organization_id: None,
            client: Client::builder()
                .default_headers(header_map)
                .build()
                .unwrap(),
            api_dict,
        }
    }

    pub fn organization_id(mut self, organization_id: String) -> Self {
        self.organization_id = Some(organization_id);
        self
    }

    pub async fn get_model(&self) -> ApiResult<ModelResponse> {
        let url = self.assemble_url("listModels");
        let res = self.client.get(&url);
        let res = if let Some(organization_id) = &self.organization_id {
            res.header("OpenAI-Organization", organization_id)
        } else {
            res
        };

        let res = res.send().await?;

        // if status is ok, return as Response json, otherwise return as ApiError
        if res.status().is_success() {
            let res = res.json::<ModelResponse>().await?;
            Ok(res)
        } else {
            let err = res.json::<GPTErrorResponse>().await?;
            Err(Error::ApiError(err))
        }
    }

    fn assemble_url(&self, api_key: &str) -> String {
        let url = format!("{}{}", Self::BASE_URL, self.api_dict.get(api_key).unwrap());
        url
    }
    async fn build(
        &self,
        api_key: &str,
        method: ApiMethod,
        payload: Option<impl Serialize>,
    ) -> ApiResult<Response> {
        let url = self.assemble_url(api_key);
        let res = match method {
            ApiMethod::GET => self.client.get(&url),
            ApiMethod::POST => self.client.post(&url).json(&payload),
        };

        let res = if let Some(organization_id) = &self.organization_id {
            res.header("OpenAI-Organization", organization_id)
        } else {
            res
        };
        let res = res.send().await?;

        //if status is ok, return as Response json, otherwise return as ApiError
        if res.status().is_success() {
            println!("{:?}", &res);
            match api_key {
                "createCompletion" => Ok(Response::Completion(
                    res.json::<CompletionResponse>().await?,
                )),
                "createChat" => Ok(Response::Chat(res.json::<ChatResponse>().await?)),
                "createEmbedding" => {
                    Ok(Response::Embedding(res.json::<EmbeddingResponse>().await?))
                }
                _ => panic!("not implemented"),
            }
        } else {
            let err = res.json::<GPTErrorResponse>().await?;
            Err(Error::ApiError(err))
        }
    }
    pub async fn chat(&self, payload: ChatPayload) -> ApiResult<ChatResponse> {
        let res = self
            .build("createChat", ApiMethod::POST, Some(payload))
            .await?;
        match res {
            Response::Chat(res) => Ok(res),
            _ => panic!("not implemented"),
        }
    }

    // Note: Embedding api has a 3500 RPM limitation
    pub async fn embedding(&self, payload: &EmbeddingPayload) -> ApiResult<EmbeddingResponse> {
        let res = self
            .build("createEmbedding", ApiMethod::POST, Some(payload))
            .await?;
        match res {
            Response::Embedding(res) => Ok(res),
            _ => panic!("not implemented"),
        }
    }

    pub async fn completion(&self, payload: &CompletionPayload) -> ApiResult<CompletionResponse> {
        let res = self
            .build("createCompletion", ApiMethod::POST, Some(payload))
            .await?;
        match res {
            Response::Completion(res) => Ok(res),
            _ => panic!("not implemented"),
        }
    }
}

#[cfg(test)]
mod api_test {
    use super::super::datamap::*;
    use super::*;
    use anyhow::Result;
    use dotenv::dotenv;
    use std::env;
    #[tokio::test]
    async fn test_gpt_get_model() -> Result<()> {
        dotenv().ok();
        let api_secret = env::var("GPT_TOKEN").unwrap_or("no GPT token".to_string());
        let token = Token::new(api_secret);
        let request = Api::new(token);

        let res = request.get_model().await;
        println!("{:?}", res);

        Ok(())
    }
    #[tokio::test]
    async fn test_gpt_chat() -> Result<()> {
        dotenv().ok();
        let api_secret = env::var("GPT_TOKEN").unwrap_or("no GPT token".to_string());
        let token = Token::new(api_secret);
        let request = Api::new(token);

        // chat payload
        let msg: Message = Message {
            role: "user".to_string(),
            content: "Hello to you, gpt turbo!".to_string(),
        };
        let chat_payload: ChatPayload = ChatPayload {
            model: Model::Gpt35Turbo,
            messages: vec![msg],
            ..Default::default()
        };
        let res = request.chat(chat_payload).await?;
        println!("{:?}", res);

        Ok(())
    }

    // unit test for function embedding in Api struct
    #[tokio::test]
    async fn test_embedding() -> Result<()> {
        dotenv().ok();
        // Initialize an instance of the ChatClient
        let api_secret = env::var("GPT_TOKEN").unwrap_or("no GPT token".to_string());
        let token = Token::new(api_secret);
        let request = Api::new(token);
        // the embedding vectors always has 1536 length.

        // Create an EmbeddingPayload object
        let payload = EmbeddingPayload {
            model: Model::Ada002,
            input: "This is an embedding test".to_string(),
            ..Default::default()
        };
        let res = request.embedding(&payload).await?;
        // Call the embedding function

        // Check if the response has been successful

        Ok(())
    }

    #[tokio::test]
    async fn test_completion() -> Result<()> {
        dotenv().ok();
        // Initialize an instance of the ChatClient
        let api_secret = env::var("GPT_TOKEN").unwrap_or("no GPT token".to_string());
        let token = Token::new(api_secret);
        let request = Api::new(token);
        // the embedding vectors always has 1536 length.

        // Create an EmbeddingPayload object
        let payload = CompletionPayload {
            model: Model::Davinci003,
            prompt: "Summarize this for a second-grade student:
            Jupiter is the fifth planet from the Sun and the largest in the Solar System. It is a gas giant with a mass one-thousandth that of the Sun, but two-and-a-half times that of all the other planets in the Solar System combined. Jupiter is one of the brightest objects visible to the naked eye in the night sky, and has been known to ancient civilizations since before recorded history. It is named after the Roman god Jupiter.[19] When viewed from Earth, Jupiter can be bright enough for its reflected light to cast visible shadows,[20] and is on average the third-brightest natural object in the night sky after the Moon and Venus.".to_string(),
            temperature: Some(0.7),
            max_tokens: Some(256),
            top_p: Some(1.0),
            frequency_penalty: Some(0.0),
            presence_penalty: Some(0.0),
            ..Default::default()
        };
        let res = request.completion(&payload).await?;

        println!("{:?}", res);
        // Call the embedding function

        // Check if the response has been successful

        Ok(())
    }
}
