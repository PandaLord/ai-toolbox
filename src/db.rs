use sled::{Db, Error, Result,IVec,Tree};
use crate::datamap::{Conversation, Message, Sentances};
use uuid::Uuid;
use std::str;
use serde::{Deserialize, Serialize};
use anyhow::{anyhow};
use bincode;
// 初始化数据库
pub fn initialize_database() -> Db {
    let db = sled::open("init").expect("Failed to open sled database");
    db
}
pub fn init() {
    let db = sled::open("./init").expect("Failed to open sled database");

}

// 更新数据库
// pub fn update_database(
//     db: &Db,
//     conversation: &Conversation,
//     messages: &[&Sentances; 2],
// ) -> Result<()> {
//     let conversation_json = serde_json::to_string(&conversation).unwrap();
//     db.insert(&conversation.conversation_id, conversation_json.as_bytes())?;
//
//     for message in messages {
//         let message_json = serde_json::to_string(&message).unwrap();
//         let message_key = format!("{}:{}", conversation.conversation_id, message.sentance_id);
//         db.insert(&message_key, message_json.as_bytes())?;
//     }
//
//     Ok(())
// }
pub fn update_database(
    db: &Db,
    conversation: &Conversation,
    messages: &[&Sentances],
) -> Result<()> {
    let conversation_json = serde_json::to_string(&conversation).unwrap();
    db.insert(&conversation.conversation_id, conversation_json.as_bytes())?;

    for message in messages {
        let message_json = serde_json::to_string(&message).unwrap();
        let message_key = format!("{}:{}", conversation.conversation_id, message.sentance_id);
        db.insert(&message_key, message_json.as_bytes())?;
    }

    Ok(())
}


// 查询数据库中的对话内容
// pub fn query_conversation(
//     db: &Db,
//     conversation_id: &str,
//     message_count: u64,
// ) -> Result<Vec<Sentances>> {
//     let mut messages = vec![];
//
//     for message_id in 1..=message_count {
//         let message_key = format!("{}:{}", conversation_id, message_id);
//         let message_data = db.get(&message_key)?.unwrap();
//         let message: Sentances = serde_json::from_slice(&message_data).unwrap();
//         messages.push(message);
//     }
//
//     Ok(messages)
// }
pub fn query_conversation(
    db: &Db,
    conversation_id: &str,
) -> Result<Vec<Sentances>> {
    let mut messages = vec![];
    let mut message_id = 1;

    loop {
        let message_key = format!("{}:{}", conversation_id, message_id);
        match db.get(&message_key)? {
            Some(message_data) => {
                let message: Sentances = serde_json::from_slice(&message_data).unwrap();
                messages.push(message);
                message_id += 1;
            }
            None => break,
        }
    }

    Ok(messages)
}



// 将 Sentances 转换为 Message
pub fn convert_to_messages(sentances: Vec<Sentances>) -> Vec<Message> {
    sentances.into_iter().map(|sentance| {
        Message {
            role: sentance.sender,
            content: sentance.content,
        }
    }).collect()
}

#[test]
fn test() -> Result<()> {
    // 初始化数据库
    let db = initialize_database();

    // 创建一个新的对话
    let conversation_id = "1".to_string();
    let conversation = Conversation {
        conversation_id: conversation_id.clone(),
        created_at: "2023-03-15T10:00:00Z".to_string(),
        updated_at: "2023-03-15T10:00:00Z".to_string(),
        user_id: "12345".to_string(),
    };

    // 用户发送消息
    let user_message = Sentances {
        sentance_id: 1,
        conversation_id: conversation_id.clone(),
        sender: "user".to_string(),
        content: "你好，GPT-3！".to_string(),
        timestamp: "2023-03-15T10:00:30Z".to_string(),
    };

    // GPT-3 回复
    let gpt3_response = "你好！请问有什么问题我可以帮您解答？".to_string();

    let gpt3_message = Sentances {
        sentance_id: 2,
        conversation_id: conversation_id.clone(),
        sender: "GPT-3".to_string(),
        content: gpt3_response,
        timestamp: "2023-03-15T10:01:00Z".to_string(),
    };

    // 更新数据库
    update_database(&db, &conversation, &[&user_message, &gpt3_message])?;

    // 查询数据库中的对
    // 查询数据库中的对话内容
    let messages = query_conversation(&db, &conversation_id)?;

    // 显示对话内容
    println!("对话内容：");
    for message in messages {
        println!("[{}][{}]: {}", message.timestamp, message.sender, message.content);
    }
    Ok(())
}
#[test]
fn test2(){
    let db = sled::open("init").expect("Failed to open sled database");
    let messages = query_conversation(&db, "1").unwrap();

    // 显示对话内容
    println!("对话内容：");
    for message in messages {
        println!("[{}][{}]: {}", message.timestamp, message.sender, message.content);
    }
}