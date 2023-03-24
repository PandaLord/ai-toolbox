use crate::api::*;
use crate::datamap::*;
use crate::db::*;
use chrono::Utc;
use crate::api::*;
use dotenv::dotenv;
use reqwest::{
    header::HeaderMap,
    header::{AUTHORIZATION, CONTENT_TYPE},
    Client,
};
use reqwest::Error as ReqwestError;
use serde_json::Value;
use std::{collections::HashMap, env, fs};
use uuid::Uuid;
// use anyhow::Error;
// use crate::gpt::error::GPTErrorResponse;
use sled::{Db, Error,IVec, Result, Tree};
use crate::error::*;

use super::{
    datamap::{ChatPayload, ChatResponse, ModelResponse},
    error::ApiResult,
    token::Token,
};


pub async fn interact_with_gpt(request: &Api, conversation_id: &str, db: &Db) -> std::result::Result<Sentances, CustomError>{
    // let db = sled::open("./init").expect("Failed to open sled database");

    // Query conversation messages from the database
    let messages = query_conversation(&db, &conversation_id)?;

    // Convert Sentances to Message for GPT-3 interaction
    let gpt_messages = convert_to_messages(messages);

    // Prepare the chat payload
    let chat_payload: ChatPayload = ChatPayload {
        model: Model::Gpt35Turbo,
        messages: gpt_messages,
        ..Default::default()
    };

    // Send the request to GPT-3
    // let res = match request.chat(chat_payload).await {
    //     Ok(res) => res,
    //     Err(err) => err,
    // };
    // let res = request.chat(chat_payload).await
    //     .map_err(|err| sled::Error(err))?;
    //
    // let res = request.chat(chat_payload).await?;
// Send the request to GPT-3
    let res = request.chat(chat_payload).await.map_err(CustomError::GPTApiError)?;

    //解码 reqwest::Response 时出现了问题，导致 Result 的 Err 分支被执行。
    // 具体地，解码器找到了一个未知的变量 message，而期望的是 ChatError 或 EditError。

    // Get the GPT-3 response
    let gpt3_response = res.choices[0].message.content.clone();

    // Create a Sentances object for GPT-3's response
    let gpt3_message = Sentances {
        sentance_id: query_conversation(&db, &conversation_id)?.len() as u64 + 1,
        conversation_id: conversation_id.to_string(),
        sender: "assistant".to_string(),
        content: gpt3_response,
        timestamp: Utc::now().to_rfc3339(),
    };

    // Retrieve the conversation from the database
    let conversation_data = db.get(conversation_id)?.expect("Conversation not found");
    let mut conversation: Conversation = serde_json::from_slice(&conversation_data).unwrap();

    // Update the conversation.updated_at timestamp
    conversation.updated_at = Utc::now().to_rfc3339();

    // Update the database with the new GPT-3 message
    update_database(&db, &conversation, &[&gpt3_message])?;

    Ok(gpt3_message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpt::*;
    use std::env;

    #[tokio::test]
    async fn test_interact_with_gpt() -> std::result::Result<(), CustomError> {
        dotenv().ok();
        let api_secret = env::var("GPT_TOKEN").unwrap_or("no GPT token".to_string());
        let token = Token::new(api_secret);
        let request = Api::new(token);

        let db = sled::open("init").expect("Failed to open sled database");

        let conversation_id = "1".to_string();
        let conversation = Conversation {
            conversation_id: conversation_id.clone(),
            created_at: "2023-03-15T10:00:00Z".to_string(),
            updated_at: "2023-03-15T10:00:00Z".to_string(),
            user_id: "12345".to_string(),
        };

        let user_message = Sentances {
            sentance_id: 1,
            conversation_id: conversation_id.clone(),
            sender: "user".to_string(),
            content: "ok you are better than google".to_string(),
            timestamp: "2023-03-15T10:00:30Z".to_string(),
        };

        // Add a second user message
        let user_message2 = Sentances {
            sentance_id: 2,
            conversation_id: conversation_id.clone(),
            sender: "user".to_string(),
            content: "Tell me a joke.".to_string(),
            timestamp: "2023-03-15T10:01:30Z".to_string(),
        };
        update_database(&db, &conversation, &[&user_message,&user_message2])?;
        // update_database(&db, &conversation, &[&user_message2])?;

        // Interact with GPT-3 for the first user message
        interact_with_gpt(&request, &conversation_id, &db).await?;

        // Interact with GPT-3 for the second user message
        interact_with_gpt(&request, &conversation_id, &db).await?;

        let messages = query_conversation(&db, &conversation_id)?;

        println!("Messages in the conversation:");
        for message in messages {
            println!("{}: {}", message.sender, message.content);
        }

        Ok(())
    }
}

#[test]
fn test_is_in_db(){
    let db = sled::open("./init").expect("Failed to open sled database");

    let messages = query_conversation(&db, "1").unwrap();

    // Convert Sentances to Message for GPT-3 interaction
    let gpt_messages = convert_to_messages(messages);
    println!("{:?}", gpt_messages);
}
