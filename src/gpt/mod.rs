// mod notion;
// mod notion_payload;
// pub mod query;
mod gpt;
mod chat;






use gpt::*;
use self::chat::ChatFormat;


use anyhow::Result;
use dotenv::dotenv;
// pub use notion::*;
use reqwest::{
    header::{HeaderMap, CONTENT_TYPE},
    Client, RequestBuilder,
};
use serde::{Serialize, Deserialize};

use std::{collections::HashMap, env, fs};
use thiserror::Error;

use crate::utils::format_url;



// use serde_json::{Result, Value};
// use crate::format_url;

const API_PATH: &'static str = "./src/gpt/api.json";

// define common trait for create requests



pub struct GPTRequestBuilder {
    token: String,
    api_dict: HashMap<String, String>,
}

pub trait GPTPayload {
    fn to_json(&self) -> Result<String, anyhow::Error>;
}

#[derive(Error, Debug)]
pub enum GPTRequestError {
    #[error("get request build error")]
    GetRequestError(String),
    #[error("post request build error")]
    PostRequestError(String),
    #[error("unknown request error")]
    Unknown,
}

impl Default for GPTRequestBuilder {
    fn default() -> Self {
        dotenv().ok();
        let token = env::var("GPT_TOKEN").unwrap_or("no GPT token".to_string());
        let json_data = fs::read_to_string(API_PATH).expect("unable to read Api Json");

        let res: HashMap<String, String> =
            serde_json::from_str(&json_data).expect("unable to transfer to hashmap");

        Self {
            token,
            api_dict: res,
        }
    }
}

impl GPTRequestBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    fn retrieve_url(&self, api_name: &str) -> Option<String> {
        let base_url = self.api_dict.get("baseUrl");
        let api_url = self.api_dict.get(api_name);
        if base_url.is_some() && api_url.is_some() {
            let result = base_url.unwrap().clone() + api_url.unwrap();
            Some(result)
        } else {
            None
        }
    }

    fn basic_request_header() -> HeaderMap {
        let mut header_map = HeaderMap::new();
        header_map.insert(CONTENT_TYPE, "application/json".parse().unwrap());
        header_map
    }

    pub fn get(&self, api_name: String, params: String) -> Result<RequestBuilder, GPTRequestError> {
        let url = self.retrieve_url(&api_name);
        let headers = GPTRequestBuilder::basic_request_header();
        if let Some(url) = url {
            // TODO: remove format url
            let full_url = format_url(&url, params);
            println!("{}", full_url);
            let request_builder = Client::new()
                .get(full_url)
                .bearer_auth(&self.token)
                .headers(headers);
            Ok(request_builder)
        } else {
            Err(GPTRequestError::GetRequestError("test".to_string()))
        }
    }

    pub fn post(
        &self,
        api_name: String,
        params: String,
        payload: String,
    ) -> Result<RequestBuilder, GPTRequestError> {
        let url = self.retrieve_url(&api_name);
        let headers = GPTRequestBuilder::basic_request_header();
        if let Some(url) = url {
            // TODO: remove format url
            let full_url = format_url(&url, params);
            let request = Client::new()
                .post(full_url)
                .bearer_auth(&self.token)
                .headers(headers)
                .body(payload);
            Ok(request)
        } else {
            Err(GPTRequestError::PostRequestError("test".to_string()))
        }
    }
}
