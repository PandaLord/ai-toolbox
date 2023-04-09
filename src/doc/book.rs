use std::{
    env,
    fs::{self, File},
    io::{BufRead, BufReader, Read},
    iter::zip,
    path::Path,
    sync::{Arc, Mutex},
};

use crate::{
    api::Api,
    datamap::{
        ChatPayload, EmbeddingData, EmbeddingPayload, Message, Model, Usage, ApiResponse, UsageReport,
    },
    token::Token,
    PointMetadata, QdrantDb,
};
use anyhow::Result;
use dotenv::dotenv;
use encoding_rs::GBK;
use qdrant_client::qdrant::{r#match::MatchValue, value::Kind, FieldCondition, Filter, Match};
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use tokio::sync::Semaphore;

#[derive(Default, Serialize, Deserialize)]
pub struct Book {
    name: String,
    id: Uuid,
    content: Vec<String>,
    #[serde(skip_serializing, skip_deserializing)]
    _api_client: Option<Api>,
    #[serde(skip_serializing, skip_deserializing)]
    _embedding_ready: bool,
    #[serde(skip_serializing, skip_deserializing)]
    _qdrant_client: Option<QdrantDb>,
}

impl Book {
    pub fn new(name: String) -> Self {
        Self {
            name,
            id: Uuid::new_v4(),
            content: Vec::new(),
            ..Default::default()
        }
    }

    pub fn upload<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        let ext = path.extension().expect("no extension");
        let mut book = Self::new(path.file_name().unwrap().to_str().unwrap().to_string());
        let mut temp_content = vec![];

        if ext == "txt" {
            info!("Start parsing Txt file");
            let content = Self::resolve_txt(path);
            book.content = content.unwrap();
            info!("Parsing finished, contents: {:?}", &book.content[..100]);
        } else {
            info!("Start parsing Md file");
            let file = File::open(path).unwrap();
            let reader = BufReader::new(file);
            for line in reader.lines() {
                if line.as_ref().unwrap() != "" {
                    temp_content.push(line.unwrap());
                }
                if temp_content.len() >= 10 {
                    book.content.push(temp_content.join("\r\n"));
                    temp_content = vec![];
                }
            }
        }

        book
    }
    // TODO:有一些txt无法parse为utf8
    fn resolve_txt(path: &Path) -> Result<Vec<String>, String> {
        // 1. 读取文件内容
        // let file_content = fs::read(path).expect("Failed to read file");
        let mut file = File::open(path).expect("Failed to read file");
        let mut file_content = Vec::new();
        file.read_to_end(&mut file_content)
            .expect("Failed to read file contents");
        let (cow, _, had_errors) = GBK.decode(&file_content);
        if had_errors {
            return Err("Had errors when parsing file".to_string());
        } else {
            let utf8_content = cow.into_owned();
            let utf8_content = utf8_content.replace("\u{3000}", "");
            // println!("{}", String::from_utf8_lossy(&utf8_content));
            // let contents = String::from_utf8_lossy(&file_content).into_owned();
            let content_vec: Vec<String> =
                utf8_content.split("\r\n").map(|e| e.to_string()).collect();
            let fixed_len = 100;
            info!(
                "Reading file contents, lens: {}, section size: {}",
                utf8_content.len(),
                fixed_len
            );
            //

            // let mut result = Vec::new();
            // let mut buf = String::new();
            // info!("Parsing...");
            // for s in content_vec.iter() {
            //     buf.push_str(s);
            //     if buf.len() >= fixed_len {
            //         result.push(buf.clone());
            //         buf.clear();
            //     }
            // }
            // if buf.len() > 0 {
            //     result.push(buf.clone());
            // }

            return Ok(content_vec);
        }
    }

    pub fn init_api(&mut self) {
        dotenv().ok();
        let api_secret = env::var("GPT_TOKEN").unwrap_or("no GPT token".to_string());
        let token = Token::new(api_secret);
        let request = Api::new(token);
        self._api_client = Some(request);
    }

    pub async fn init_db(&mut self) {
        let qdrant = QdrantDb::new().await;
        self._qdrant_client = Some(qdrant);
    }

    // TODO: batch embedding
    pub async fn update_embedding(&mut self) -> Result<()> {
        // chat init
        // Initialize an instance of the ChatClient
        if self._api_client.is_none() {
            self.init_api();
            info!("init Api Client");
        }
        // qdrant init
        if self._qdrant_client.is_none() {
            self.init_db().await;
            info!("init Qdrant Client");
        }

        let c_id = Uuid::new_v4();

        // Create an EmbeddingPayload object
        let embedding_payloads: Vec<EmbeddingPayload> = self
            .content
            .iter()
            .map(|line| EmbeddingPayload {
                model: Model::Ada002,
                input: line.clone(),
                ..Default::default()
            })
            .collect();
        let metadata_vec: Vec<PointMetadata> = self
            .content
            .iter()
            .map(|line| PointMetadata {
                conversation_id: c_id.clone(),
                raw_content: line.clone(),
                book_id: Some(self.id.clone()),
                ..Default::default()
            })
            .collect();

        let embedding_vector: Arc<Mutex<Vec<EmbeddingData>>> = Arc::new(Mutex::new(vec![]));
        let usage_vector: Arc<Mutex<Vec<Usage>>> = Arc::new(Mutex::new(vec![]));

        let total_count = embedding_payloads.len();
        info!(
            "start getting embedding from openai, request counts: {}",
            total_count
        );
        let mut handles = Vec::new();
        let count: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));
        for payload in embedding_payloads {
            let client = self._api_client.as_ref().unwrap().to_owned();
            let embedding_vector = Arc::clone(&embedding_vector);
            let count = Arc::clone(&count);
            let usage_vector = Arc::clone(&usage_vector);

            let handle = tokio::spawn(async move {
                let res = client.embedding(&payload).await;
                match res {
                    Ok(res) => {
                        let mut count = count.lock().unwrap();
                        let mut usage_vector = usage_vector.lock().unwrap();
                        usage_vector.push(res.log());
                        *count += 1;
                        info!(
                            "embedding count: {}, left: {}",
                            count,
                            total_count as i32 - *count
                        );
                        let mut embedding_vector = embedding_vector.lock().unwrap();
                        embedding_vector.push(res.data[0].clone());
                    }
                    Err(e) => {
                        panic!("embedding error: {:?}", e);
                    }
                }
            });
            handles.push(handle);
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        info!("handles assemble finished");
        futures::future::join_all(handles).await;
        info!("Finished pushing embedding from openai..");
        let usage_vector = Arc::try_unwrap(usage_vector).unwrap().into_inner().unwrap();
        let usage_report = UsageReport(usage_vector.clone());
        let metadata_vec: Vec<PointMetadata> = zip(usage_vector, metadata_vec)
            .map(|(usage, metadata)| PointMetadata {
                token_count: usage.total_tokens,
                ..metadata
            })
            .collect();

        info!("{}", usage_report);

        info!("Start upserting points to qdrant..");

        let embedding_vector = Arc::try_unwrap(embedding_vector)
            .unwrap()
            .into_inner()
            .unwrap();

        info!(
            "embedding vector len: {}, metadata_vec: {}",
            embedding_vector.len(),
            metadata_vec.len()
        );
        self._qdrant_client
            .as_ref()
            .unwrap()
            .upsert_points(embedding_vector, "gpt_embeddings", metadata_vec)
            .await?;
        info!("Finished upserting points to qdrant..");
        self._embedding_ready = true;
        Ok(())
    }
    pub async fn query_book_with_chat(
        &mut self,
        question: &str,
        book_id: String,
    ) -> Result<String> {
        if self._api_client.is_none() {
            self.init_api();
        }
        // qdrant init
        if self._qdrant_client.is_none() {
            self.init_db().await;
        }
        // get current embeddings
        // Create an EmbeddingPayload object
        let embedding_payload = EmbeddingPayload {
            model: Model::Ada002,
            input: question.to_string(),
            ..Default::default()
        };
        let res = self
            ._api_client
            .as_ref()
            .unwrap()
            .embedding(&embedding_payload)
            .await?;
        let current_embedding = res.data.get(0).unwrap();
        let collection_name = "gpt_embeddings";
        let mut filter = Filter {
            should: vec![],
            must: vec![],
            must_not: vec![],
        };
        filter.should.push(
            FieldCondition {
                key: "book_id".to_string(),
                r#match: Some(Match {
                    // match_value: Some(MatchValue::Keyword(self.id.to_string())),
                    match_value: Some(MatchValue::Text(book_id)),
                }),
                ..Default::default()
            }
            .into(),
        );
        let relevant_data = self
            ._qdrant_client
            .as_ref()
            .unwrap()
            .query_points(collection_name, current_embedding, filter)
            .await?;

        println!("relevant data response: {:?}", relevant_data);

        let context: Vec<String> = relevant_data
            .result
            .iter()
            .map(|x| {
                if let Some(raw_content) = x.payload.get("raw_content") {
                    if let Some(kind) = raw_content.kind.as_ref() {
                        match kind {
                            Kind::StringValue(data) => data.to_owned(),
                            _ => "".to_string(),
                        }
                    } else {
                        "".to_string()
                    }
                } else {
                    "".to_string()
                }
            })
            .collect();

        let f_question = "我会在接下来的内容中提供一份文章内容,请你仔细理解文章,我会基于文章内容向你提问,请你完全基于文章内容回答我,如果无法基于文章内容回答则回答无可奉告. 文章内容: ".to_string()
            + "'''"
            + context.join("").as_str()
            + "'''"
            + "\r\n"
            + "问题: "
            + question;

        println!("final question: {:?}", &f_question);

        let msg: Message = Message {
            role: "user".to_string(),
            content: f_question.clone(),
        };
        let chat_payload: ChatPayload = ChatPayload {
            model: Model::Gpt35Turbo,
            messages: vec![msg],
            ..Default::default()
        };

        let res_chat = self
            ._api_client
            .as_ref()
            .unwrap()
            .chat(chat_payload)
            .await?;
        Ok(res_chat.to_string())
        // chat payload
        // let msg: Message = Message {
        //     role: "user".to_string(),
        //     content: "Hello to you, gpt turbo!".to_string(),
        // };
        // let chat_payload: ChatPayload = ChatPayload {
        //     model: Model::Gpt35Turbo,
        //     messages: vec![msg],
        //     ..Default::default()
        // };
    }
}

#[cfg(test)]
mod book {

    use super::*;
    use tracing_test::traced_test;
    #[tokio::test]
    #[traced_test]
    // create a test which could upload a file and get the relevant section feedback for this file
    async fn test_upload() {
        // embedding and chat should separate.
        let mut book = Book::upload("assets/law.md");
        book.update_embedding().await.unwrap();
        let res = book
            .query_book_with_chat("妇女和男人的就业权力一样么?", book.id.to_string())
            .await
            .unwrap();
        println!("res: {:?}", res);
    }
    #[tokio::test]
    async fn test_txt_uplaod() {
        // embedding and chat should separate.
        let mut book = Book::upload("assets/test.txt");
        // book.update_embedding().await.unwrap();
        let res = book
            .query_book_with_chat("詹岚是一个什么样的人?", book.id.to_string())
            .await
            .unwrap();
        println!("res: {:?}", res);
    }
    #[tokio::test]
    #[traced_test]
    async fn test_txt_upload() {
        // embedding and chat should separate.
        let mut book = Book::upload("assets/连城诀.txt");
        // book.update_embedding().await.unwrap();
        let res = book
            // .query_book_with_chat("这篇文章主要内容是什么", book.id.to_string())
            .query_book_with_chat("这篇文章主要内容是什么", "b8506821-2b49-4b8e-886e-c10abd71f496".to_string())
            .await
            .unwrap();
        println!("res: {:?}", res);
    }

    #[tokio::test]
    #[traced_test]
    async fn test_embedding() {
        // 连城诀 - ddbea9ad-bed2-4dd9-a8c2-16997828ddf9
        // 射雕英雄传 - ab678c31-032a-4d5e-b609-aa3599d061e6
        let mut book = Book::upload("assets/射雕英雄传.txt");
        book.update_embedding().await.unwrap();
    }
}
