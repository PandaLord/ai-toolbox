use dotenv::dotenv;
use reqwest::{
    header::HeaderMap,
    header::{AUTHORIZATION, CONTENT_TYPE},
    Client,
};
use std::{collections::HashMap, env, fs};
use crate::gpt::error::{Error, GPTErrorResponse};
use super::{
    datamap::{ChatPayload, ChatResponse, ModelResponse},
    error::ApiResult,
    token::Token,
};

// abstract Api struct to call different apis
pub struct Api {
    organization_id: Option<String>,
    client: Client,
    api_dict: HashMap<String, String>,
}

impl Api {
    const BASE_URL: &'static str = "https://api.openai.com/v1";
    const API_PATH: &'static str = "./src/gpt/api.json";

    pub fn new(key: Token) -> Api {
        let mut header_map = HeaderMap::new();
        let json_data: String =
            fs::read_to_string(Self::API_PATH).expect("unable to read Api Json");
        let api_dict: HashMap<String, String> =
            serde_json::from_str(&json_data).expect("unable to transfer to hashmap");

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

    pub async fn chat(&self, payload: ChatPayload) -> ApiResult<ChatResponse> {
        const CHAT_API_KEY: &str = "createChat";
        let url = format!(
            "{}{}",
            Self::BASE_URL,
            self.api_dict.get(CHAT_API_KEY).unwrap()
        );
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
        // pub async fn get_model(&self) {
        const CHAT_API_KEY: &str = "listModels";
        let url = format!(
            "{}{}",
            Self::BASE_URL,
            self.api_dict.get(CHAT_API_KEY).unwrap()
        );
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


}



#[cfg(test)]
mod api_test {
    use super::super::datamap::*;
    use super::*;
    use anyhow::Result;
    use dotenv::dotenv;
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
    // #[tokio::test]
    // async fn test_gpt_chat() -> Result<()> {
    //     dotenv().ok();
    //     let api_secret = env::var("GPT_TOKEN").unwrap_or("no GPT token".to_string());
    //     let token = Token::new(api_secret);
    //     let request = Api::new(token);
    //
    //     let db = sled::open("./init").expect("Failed to open sled database");
    //     // 创建一个新的对话
    //     let conversation_id = Uuid::new_v4().to_string();
    //     let conversation = Conversation {
    //         conversation_id: conversation_id.clone(),
    //         created_at: "2023-03-15T10:00:00Z".to_string(),
    //         updated_at: "2023-03-15T10:00:00Z".to_string(),
    //         user_id: "12345".to_string(),
    //     };
    //     // 用户发送消息
    //     let user_message = Sentances {
    //         sentance_id: 1,
    //         conversation_id: conversation_id.clone(),
    //         sender: "user".to_string(),
    //         content: "你是一个非常有帮助的人工智能助手，旨在帮我解决一切问题。".to_string(),
    //         timestamp: "2023-03-15T10:00:30Z".to_string(),
    //     };
    //     // GPT-3 回复
    //     let gpt3_response = "感谢夸奖，我会尽力为你解决问题".to_string();
    //
    //     let gpt3_message = Sentances {
    //         sentance_id: 2,
    //         conversation_id: conversation_id.clone(),
    //         sender: "assistant".to_string(),
    //         content: gpt3_response,
    //         timestamp: "2023-03-15T10:01:00Z".to_string(),
    //     };
    //     // 更新数据库
    //     update_database(&db, &conversation, &[&user_message, &gpt3_message]).unwrap();
    //
    //     let msg = convert_to_messages(query_conversation(&db, &conversation_id)?);
    //     println!("msg:{:?}",msg);
    //     let chat_payload: ChatPayload = ChatPayload {
    //         model: Model::Gpt35Turbo,
    //         messages: msg,
    //         ..Default::default()
    //     };
    //
    //     let res = request.chat(chat_payload).await?;
    //     println!("{:?}", res);
    //
    //     Ok(())
    // }
}
