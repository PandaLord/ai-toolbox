use std::{env, collections::HashMap, fs};
use dotenv::dotenv;
use reqwest::{Client, header::HeaderMap, header::{CONTENT_TYPE, AUTHORIZATION}};

use crate::gpt::error::{GPTErrorResponse, Error};

use super::{token::Token, datamap::{ChatPayload, ChatResponse, ModelResponse}, error::ApiResult};


// abstract Api struct to call different apis
pub struct Api {
    organization_id: Option<String>,
    client: Client,
    api_dict: HashMap<String, String>
}



impl Api {
    const BASE_URL: &'static str = "https://api.openai.com/v1";
    const API_PATH: &'static str = "./src/gpt/api.json";

    pub fn new(key: Token) -> Api {
        
        let mut header_map = HeaderMap::new();
        let json_data: String = fs::read_to_string(Self::API_PATH).expect("unable to read Api Json");
        let api_dict: HashMap<String, String> =
            serde_json::from_str(&json_data).expect("unable to transfer to hashmap");
        
        // header_map.insert(CONTENT_TYPE, "application/json".parse().unwrap());
        header_map.insert(CONTENT_TYPE, "application/json".parse().unwrap());
        header_map.insert(AUTHORIZATION, key.to_string().parse().unwrap());
        Api {
            organization_id: None, 
            client: Client::builder().default_headers(header_map).build().unwrap(),
            api_dict
        }
    }

    pub fn organization_id(mut self, organization_id: String) -> Self {
        self.organization_id = Some(organization_id);
        self
    }

    pub async fn chat(&self, payload: ChatPayload) -> ApiResult<ChatResponse> {
        const CHAT_API_KEY: &str = "createChat";
        let url = format!("{}{}", Self::BASE_URL, self.api_dict.get(CHAT_API_KEY).unwrap());
        let res = self.client.post(&url).json(&payload);
        let res = if let Some(organization_id) = &self.organization_id {
            res.header("OpenAI-Organization", organization_id)
        } else {
            res
        };

        let res = res.send().await?;

        //if status is ok, return as Response json, otherwise return as ApiError
        if res.status().is_success() {
            let res = res.json::<ChatResponse>().await?;
            Ok(res)
        } else {
            let err = res.json::<GPTErrorResponse>().await?;
            Err(Error::ApiError(err))
        }
    }

    pub async fn get_model(&self) -> ApiResult<ModelResponse> {
        const CHAT_API_KEY: &str = "listModels";
        let url = format!("{}{}", Self::BASE_URL, self.api_dict.get(CHAT_API_KEY).unwrap());
        let res = self.client.get(&url);
        let res = if let Some(organization_id) = &self.organization_id {
            res.header("OpenAI-Organization", organization_id)
        } else {
            res
        };

        let res = res.send().await?;

        let body = &res.text().await?;

        println!("{}", body);

        //if status is ok, return as Response json, otherwise return as ApiError
        if res.status().is_success() {
            let res = res.json::<ModelResponse>().await?;
            Ok(res)
        } else {
            let err = res.json::<GPTErrorResponse>().await?;
            Err(Error::ApiError(err))
        }
    }

}

#[cfg(test)]
mod api_test {
    use super::*;
    use dotenv::dotenv;
    use anyhow::Result;
    #[tokio::test]
    async fn test_gpt_get_model() -> Result<()> {
        dotenv().ok();
        let api_secret = env::var("GPT_TOKEN").unwrap_or("no GPT token".to_string());
        let token = Token::new(api_secret);
        let request = Api::new(token);

        let res = request.get_model().await?;
        println!("{:?}", res);
        // let request: GPTRequestBuilder = GPTRequestBuilder::default();
        // let builder = get_model(&request).await?;

        // if let Ok(status) = get_and_print_reponse(builder).await {
        //     assert_eq!(status, 200);
        // }
        // let response = builder.send().await?;
        // let status = response.status();
        // let body = &response.text().await?;
        // let response: Value = serde_json::from_str(body)?;
        // println!("res body: {}", body);

        Ok(())
    }
}