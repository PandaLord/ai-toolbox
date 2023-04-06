use std::{
    env,
    error::Error,
    fs::File,
    io::{BufRead, BufReader, Read},
    iter::zip,
    path::Path,
};

use crate::{
    api::Api,
    datamap::{ChatPayload, EmbeddingData, EmbeddingPayload, Message, Model},
    error::{EmbeddingError, GPTError},
    token::Token,
    PointMetadata, QdrantDb,
};
use anyhow::Result;
use dotenv::dotenv;
use encoding_rs::Encoding;
use qdrant_client::qdrant::{r#match::MatchValue, value::Kind, FieldCondition, Filter, Match};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, instrument, trace, warn};
use uuid::Uuid;
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
            book.content = content;
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

    fn resolve_txt(path: &Path) -> Vec<String> {
        // 1. 读取文件内容
        let mut file = File::open(path).expect("无法打开文件");
        let mut file_content = Vec::new();
        file.read_to_end(&mut file_content)
            .expect("无法读取文件内容");
        let contents = String::from_utf8_lossy(&file_content).into_owned();
        let content_vec: Vec<String> = contents.split("\r\n").map(|e| e.to_string()).collect();
        let fixed_len = 700;
        info!(
            "Reading file contents, lens: {}, section size: {}",
            contents.len(),
            fixed_len
        );
        //

        let mut result = Vec::new();
        let mut buf = String::new();
        info!("Parsing...");
        for s in content_vec.iter() {
            buf.push_str(s);
            if buf.len() >= fixed_len {
                result.push(buf.clone());
                buf.clear();
            }
        }
        if buf.len() > 0 {
            result.push(buf.clone());
        }

        result
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
            })
            .collect();

        let mut embedding_vector: Vec<EmbeddingData> = vec![];
        let total_count = embedding_payloads.len();
        info!("start getting embedding from openai, request counts: {}", total_count);
        let mut count = 0;
        // TODO: need to use concurrency
        for payload in embedding_payloads {
            let res = self._api_client.as_ref().unwrap().embedding(&payload).await;
            match res {
                Ok(res) => {
                    count += 1;
                    info!("embedding count: {}, left: {}", count, total_count - count);
                    embedding_vector.push(res.data[0].clone());
                },
                Err(e) => {
                    panic!("embedding error: {:?}", e);
                }
            }
        }
        info!("Finished pushing embedding from openai..");

        // upsert into qdrant with embeddings
        // need to update with batch upsert
        // for (embedding, payload) in zip(embedding_vector, metadata_vec) {
        //     qdrant
        //         .upsert_points(&embedding, "gpt_embeddings", payload)
        //         .await?;
        // }
        info!("Start upserting points to qdrant..");
        self._qdrant_client
            .as_ref()
            .unwrap()
            .upsert_points(embedding_vector, "gpt_embeddings", metadata_vec)
            .await?;
        info!("Finished upserting points to qdrant..");
        self._embedding_ready = true;
        Ok(())
    }
    pub async fn query_book_with_chat(&mut self, question: &str) -> Result<String> {
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
                    match_value: Some(MatchValue::Text(
                        "a8e172cc-53b5-417d-8090-995e1808011f".to_string(),
                    )),
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

        let f_question = "previous context: ".to_string()
            + context.join("").as_str()
            + "\n"
            + "now: "
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
    // create a test which could upload a file and get the relevant section feedback for this file
    async fn test_upload() {
        // embedding and chat should separate.
        let mut book = Book::upload("assets/law.md");
        // book.update_embedding().await.unwrap();
        let res = book
            .query_book_with_chat("如果我被公司恶意欠薪, 我该引用哪些条款?")
            .await
            .unwrap();
        println!("res: {:?}", res);
    }
    #[tokio::test]
    async fn test_txt_uplaod() {
        // embedding and chat should separate.
        let mut book = Book::upload("assets/infscare.txt");
        book.update_embedding().await.unwrap();
        let res = book
            .query_book_with_chat("中洲队有哪些成员?")
            .await
            .unwrap();
        println!("res: {:?}", res);
    }

    #[tokio::test]
    #[traced_test]
    async fn test_embedding() {
        let mut book = Book::upload("assets/test.txt");
        book.update_embedding().await.unwrap();
    }
}
