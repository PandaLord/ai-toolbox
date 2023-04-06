use crate::openai::datamap::EmbeddingData;
use anyhow::Result;
use dotenv::dotenv;
use qdrant_client::qdrant::vectors_config::Config;
use qdrant_client::qdrant::with_payload_selector::SelectorOptions;
use qdrant_client::qdrant::{
    CollectionInfo, CollectionOperationResponse, CreateCollection, Filter, ListCollectionsResponse,
    PointsOperationResponse, PointsSelector, ScrollPoints, ScrollResponse, SearchPoints,
    SearchResponse, VectorParams, VectorsConfig, WithPayloadSelector,
};
use qdrant_client::{prelude::*, qdrant};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::iter::zip;
use uuid::Uuid;
pub struct QdrantDb {
    client: QdrantClient,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct PointMetadata {
    pub conversation_id: Uuid,
    pub raw_content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub book_id: Option<Uuid>,
}

impl From<PointMetadata> for Payload {
    fn from(metadata: PointMetadata) -> Self {
        let mut payload = HashMap::new();
        payload.insert(
            "conversation_id",
            metadata.conversation_id.to_string().into(),
        );
        payload.insert("raw_content", metadata.raw_content.into());
        payload.insert("book_id", metadata.book_id.unwrap().to_string().into());
        payload.into()
    }
}

impl QdrantDb {
    pub async fn new() -> Self {
        dotenv().ok();
        let qdrant_url = std::env::var("QDRANT_URL").expect("QDRANT_URL must be set");
        let api_key = std::env::var("QDRANT_TOKEN").expect("QDRANT_API_KEY must be set");
        let mut config = QdrantClientConfig::from_url(&qdrant_url);
        config.set_api_key(&api_key);
        let client = QdrantClient::new(Some(config)).await.unwrap();
        Self { client }
    }

    // createa a qdrant client and create a collection.
    pub async fn create_collection(
        &self,
        collection_name: impl ToString,
    ) -> Result<CollectionOperationResponse> {
        let collection = self
            .client
            .create_collection(&CreateCollection {
                collection_name: collection_name.to_string(),
                vectors_config: Some(VectorsConfig {
                    config: Some(Config::Params(VectorParams {
                        size: 1536,
                        distance: Distance::Cosine.into(),
                    })),
                }),
                ..Default::default()
            })
            .await?;

        Ok(collection)
    }

    // list all collections in a qdrant client.
    pub async fn list_collections(&self) -> Result<ListCollectionsResponse> {
        let collections = self.client.list_collections().await?;
        Ok(collections)
    }

    // list collections info
    pub async fn list_collections_info(
        &self,
        collection_name: impl ToString,
    ) -> Result<CollectionInfo> {
        let collection_info = self
            .client
            .collection_info(collection_name)
            .await?
            .result
            .unwrap();
        Ok(collection_info)
    }

    // create a qdrant client and upsert a vector.
    pub async fn upsert_points(
        &self,
        data: Vec<EmbeddingData>,
        collection_name: impl ToString,
        payload: Vec<PointMetadata>,
    ) -> Result<PointsOperationResponse> {
        // payload is a metadata which can be bind with points vectors.
        // here we use this to bind our conversation id and other metadata further(like)
        // let payload: Payload = payload.into();
        let mix_data = zip(data, payload);
        let points = mix_data
            .map(|(data, payload)| {
                PointStruct::new(
                    Uuid::new_v4().to_string(),
                    data.embedding.clone(),
                    payload.into(),
                )
            })
            .collect();

        let res = self
            .client
            .upsert_points(collection_name, points, None)
            .await?;

        Ok(res)
    }
    // retrieve all points
    pub async fn retrieve_points(&self, collection_name: impl ToString) -> Result<ScrollResponse> {
        let res = self
            .client
            .scroll(&ScrollPoints {
                collection_name: collection_name.to_string(),
                limit: Some(100),
                with_payload: Some(WithPayloadSelector {
                    selector_options: Some(SelectorOptions::Enable(true)),
                }),
                ..Default::default()
            })
            .await?;

        Ok(res)
    }

    // clear all points
    pub async fn clear_points(
        &self,
        collection_name: impl ToString,
    ) -> Result<PointsOperationResponse> {
        let filter: Filter = Filter {
            should: vec![],
            must: vec![],
            must_not: vec![],
        };
        // let selector = PointsSelector;
        let res = self
            .client
            .delete_points(
                collection_name,
                &PointsSelector {
                    points_selector_one_of: Some(
                        qdrant::points_selector::PointsSelectorOneOf::Filter(filter),
                    ),
                },
                None,
            )
            .await?;
        Ok(res)
    }
    // create a qdrant client and search a vector.
    pub async fn query_points(
        &self,
        collection_name: impl ToString,
        data: &EmbeddingData,
        filter: Filter,
    ) -> Result<SearchResponse> {
        let search_result = self
            .client
            .search_points(&SearchPoints {
                collection_name: collection_name.to_string(),
                vector: data.embedding.clone(),
                filter: Some(filter),
                limit: 5,
                with_vectors: None,
                with_payload: Some(WithPayloadSelector {
                    selector_options: Some(SelectorOptions::Enable(true)),
                }),
                params: None,
                score_threshold: None,
                offset: None,
                ..Default::default()
            })
            .await?;

        Ok(search_result)
    }
}

// write multiple unit test cases for QdrantDb
#[cfg(test)]
mod db {
    use super::super::api::Api;
    use super::*;
    use crate::datamap::{ChatPayload, EmbeddingPayload, Message, Model};
    use crate::token::Token;
    use anyhow::Result;
    use qdrant_client::qdrant::r#match::MatchValue;
    use qdrant_client::qdrant::value::Kind;
    use qdrant_client::qdrant::{FieldCondition, Match};
    use std::env;
    use uuid::Uuid;
    #[tokio::test]
    async fn test_create_collection() -> Result<()> {
        let qdrant = QdrantDb::new().await;
        let collection = qdrant.create_collection("gpt_embeddings").await?;
        assert_eq!(collection.result, true);
        Ok(())
    }

    #[tokio::test]
    async fn test_list_collections() -> Result<()> {
        let qdrant = QdrantDb::new().await;
        let collections = qdrant.list_collections().await?;
        println!(" collections: {:?}", collections);
        Ok(())
    }

    #[tokio::test]
    async fn test_list_collection_info() -> Result<()> {
        let qdrant = QdrantDb::new().await;
        let collection_info = qdrant.list_collections_info("gpt_embeddings").await?;
        println!(" collection_info: {:?}", collection_info);
        Ok(())
    }

    #[tokio::test]
    async fn test_retrieve_points() -> Result<()> {
        let qdrant = QdrantDb::new().await;
        let collection_info = qdrant.retrieve_points("gpt_embeddings").await?;
        println!(" collection_info: {:?}", collection_info);
        Ok(())
    }

    #[tokio::test]
    async fn test_upsert_qdrant() -> Result<()> {
        dotenv().ok();
        // chat init
        // Initialize an instance of the ChatClient
        let api_secret = env::var("GPT_TOKEN").unwrap_or("no GPT token".to_string());
        let token = Token::new(api_secret);
        let request = Api::new(token);

        // Create an EmbeddingPayload object
        let payload = EmbeddingPayload {
            model: Model::Ada002,
            input: "Now you are a dunguen master, i am a player named jacky, you ok?".to_string(),
            ..Default::default()
        };
        let res = request.embedding(&payload).await?;

        // qdrant upsert
        let qdrant = QdrantDb::new().await;
        let u_id = Uuid::new_v4();
        let payload = PointMetadata {
            conversation_id: u_id.clone(),
            raw_content: payload.input,
            ..Default::default()
        };

        let collection_name = "gpt_embeddings";
        let embedding_data = res.data;
        let response = qdrant
            .upsert_points(embedding_data, collection_name, vec![payload])
            .await?;

        println!("response: {:?}", response);
        Ok(())
    }

    #[tokio::test]
    async fn test_query_points() -> Result<()> {
        dotenv().ok();
        // chat init
        // Initialize an instance of the ChatClient
        let api_secret = env::var("GPT_TOKEN").unwrap_or("no GPT token".to_string());
        let token = Token::new(api_secret);
        let request = Api::new(token);

        // Create an EmbeddingPayload object
        let payload = EmbeddingPayload {
            model: Model::Ada002,
            input: "i don't know what is embedding".to_string(),
            ..Default::default()
        };
        let res = request.embedding(&payload).await?;

        // qdrant upsert
        let qdrant = QdrantDb::new().await;

        let embedding_data = res.data.get(0).unwrap();
        let collection_name = "gpt_embeddings";
        let mut filter = Filter {
            should: vec![],
            must: vec![],
            must_not: vec![],
        };
        filter.must.push(
            FieldCondition {
                key: "conversation_id".to_string(),
                r#match: Some(Match {
                    match_value: Some(MatchValue::Text("1".to_string())),
                }),
                ..Default::default()
            }
            .into(),
        );

        let res = qdrant
            .query_points(collection_name, embedding_data, filter)
            .await?;

        println!("response: {:?}", res);
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_points() -> Result<()> {
        let qdrant = QdrantDb::new().await;
        let collection_name = "gpt_embeddings";
        let res = qdrant.clear_points(collection_name).await?;
        println!("response: {:?}", res);
        Ok(())
    }

    #[tokio::test]
    async fn test_continuous_conversation() -> Result<()> {
        dotenv().ok();
        // chat init
        // Initialize an instance of the ChatClient
        let api_secret = env::var("GPT_TOKEN").unwrap_or("no GPT token".to_string());
        let token = Token::new(api_secret);
        let request = Api::new(token);

        let first_raw_content: String = "i like Steve Jobs.".to_string();
        // first chat to setup the context
        // chat payload
        let msg: Message = Message {
            role: "user".to_string(),
            content: first_raw_content.clone(),
        };
        let chat_payload: ChatPayload = ChatPayload {
            model: Model::Gpt35Turbo,
            messages: vec![msg],
            ..Default::default()
        };

        let res_chat = request.chat(chat_payload).await?;
        println!("first question response {:?}", res_chat.to_string());
        // setup embedding and pass it to the qdrant
        // Create an EmbeddingPayload object
        let payload = EmbeddingPayload {
            model: Model::Ada002,
            input: first_raw_content.clone(),
            ..Default::default()
        };

        let res_embedding = request.embedding(&payload).await?;

        // qdrant upsert
        let qdrant = QdrantDb::new().await;
        let u_id = Uuid::new_v4();
        let payload = PointMetadata {
            conversation_id: u_id.clone(),
            raw_content: payload.input,
            ..Default::default()
        };

        let collection_name = "gpt_embeddings";
        let embedding_data = res_embedding.data;

        let response = qdrant
            .upsert_points(embedding_data, collection_name, vec![payload])
            .await?;
        println!("first question storage: {:?}", response);

        // second chat to get the response
        let second_raw_content: &str = "Can you tell me more about him?";

        let payload = EmbeddingPayload {
            model: Model::Ada002,
            input: second_raw_content.to_string(),
            ..Default::default()
        };

        let res_embedding = request.embedding(&payload).await?;

        let embedding_data = res_embedding.data.get(0).unwrap();

        let mut filter = Filter {
            should: vec![],
            must: vec![],
            must_not: vec![],
        };
        filter.must.push(
            FieldCondition {
                key: "conversation_id".to_string(),
                r#match: Some(Match {
                    match_value: Some(MatchValue::Text(u_id.to_string())),
                }),
                ..Default::default()
            }
            .into(),
        );
        let response = qdrant
            .query_points(collection_name, embedding_data, filter)
            .await?;

        // println!("second question query vector response: {:?}", &response);
        let context: Vec<String> = response
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
        println!("second question context: {:?}", &context);
        let second_raw_content = "previous context: ".to_string()
            + context.join("").as_str()
            + "\n"
            + "now: "
            + second_raw_content;

        println!("second question: {:?}", &second_raw_content);
        // second chat to setup the context
        // chat payload
        let msg: Message = Message {
            role: "user".to_string(),
            content: second_raw_content.clone(),
        };
        let chat_payload: ChatPayload = ChatPayload {
            model: Model::Gpt35Turbo,
            messages: vec![msg],
            ..Default::default()
        };

        let res_chat = request.chat(chat_payload).await?;
        println!("second question response {:?}", res_chat.to_string());
        Ok(())
    }
}
