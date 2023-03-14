

// mod notion;
// mod notion_payload;
// pub mod query;
pub mod gpt;
pub mod datamap;
pub mod error;
pub mod api;
pub mod token;


pub use gpt::*;
use datamap::*;
use error::*;
use api::*;

use anyhow::Result;
use dotenv::dotenv;
// pub use notion::*;
use reqwest::{
    header::{HeaderMap, CONTENT_TYPE},
    Client, RequestBuilder,
};

use std::{collections::HashMap, env, fs};
use thiserror::Error;
// use serde_json::{Result, Value};
// use crate::format_url;

const API_PATH: &'static str = "./src/gpt/api.json";

// define common trait for create requests


pub struct GPTRequestBuilder {
    token: String,
    api_dict: HashMap<String, String>,
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

// impl GPTRequestBuilder {
//     pub fn new() -> Self {
//         Default::default()
//     }

//     fn retrieve_url(&self, api_name: &str) -> Option<String> {
//         let base_url = self.api_dict.get("baseUrl");
//         let api_url = self.api_dict.get(api_name);
//         if base_url.is_some() && api_url.is_some() {
//             let result = base_url.unwrap().clone() + api_url.unwrap();
//             Some(result)
//         } else {
//             None
//         }
//     }

//     fn basic_request_header() -> HeaderMap {
//         let mut header_map = HeaderMap::new();
//         header_map.insert(CONTENT_TYPE, "application/json".parse().unwrap());
//         header_map
//     }

//     pub fn get(
//         &self,
//         api_name: String,
//         params: String,
//     ) -> Result<RequestBuilder, Error> {
//         let url = self.retrieve_url(&api_name);
//         let headers = GPTRequestBuilder::basic_request_header();
//         if let Some(url) = url {
//             // let full_url = format_url!(url, params);
//             let request_builder = Client::new()
//                 .get(url)
//                 .bearer_auth(&self.token)
//                 .headers(headers);
//             Ok(request_builder)
//         } else {
//             Err(Error::ApiError("test".to_string()))
//         }
//     }

//     pub fn post(
//         &self,
//         api_name: String,
//         params: String,
//         payload: String,
//     ) -> Result<RequestBuilder, Error> {
//         todo!();
//         // let url = self.retrieve_url(&api_name);
//         // let headers = GPTRequest::basic_notion_header();
//         // if let Some(url) = url {
//         //     let full_url = format_url!(url, params);
//         //     let request = Client::new()
//         //         .post(full_url)
//         //         .bearer_auth(&self.token)
//         //         .headers(headers)
//         //         .body(payload);
//         //     Ok(request)
//         // } else {
//         //     Err(GPTRequestError::Unknown("test".to_string()))
//         // }
//     }
// }
