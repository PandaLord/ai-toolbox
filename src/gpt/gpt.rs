
use super::{NotionRequest, NotionRequestError};
use anyhow::Result;
use reqwest::{RequestBuilder};

type AIResult = Result<RequestBuilder, NotionRequestError>;

pub async fn init_db(request: &NotionRequest, db_id: String, payload: String) -> ApiResult {
    let notion_request = request.post("createDatabase".to_string(), db_id, payload);

    notion_request
}

pub async fn init_page(request: &NotionRequest, db_id: String, payload: String) -> ApiResult {
    let notion_request = request.post("createPage".to_string(), db_id, payload);

    notion_request
}

pub async fn query_db(request: &NotionRequest, db_id: String, payload: String) -> ApiResult {
    let notion_request = request.post("queryDatabase".to_string(), db_id, payload);

    notion_request
}

pub fn notion_api(request: &NotionRequest, db_id: String) -> ApiResult {
    let notion_request = request.get("getDatabase".to_string(), db_id);

    notion_request
}

pub async fn get_user_info(request: &NotionRequest) -> ApiResult {
    let notion_request = request.get("WhoAmi".to_string(), "".to_owned());

    notion_request
}

// #[cfg(test)]
// mod notion_test {
//     use serde_json::{json};
//     use crate::utils::helper::get_and_print_reponse;
//     use crate::notion::notion_payload::{NotionPayload, ParentType};
//     use crate::notion::query::QueryFilter;
//     use super::*;

    

//     #[tokio::test]
//     async fn test_notion_api() -> Result<()> {
//         let database_id = "c5fe9839463a447f9f305d0d1d510f3f".to_string();
//         let request: NotionRequest = NotionRequest::default();
//         let builder = notion_api(&request, database_id)?;

//         if let Ok(status) = get_and_print_reponse(builder).await {
//             assert_eq!(status, 200);
//         }
//         // let response = builder.send().await?;
//         // let status = response.status();
//         // let body = &response.text().await?;
//         // let response: Value = serde_json::from_str(body)?;
//         // println!("res body: {}", body);

//         Ok(())
//     }

//     #[tokio::test]
//     async fn query_db_test() -> Result<()> {
//         let request: NotionRequest = NotionRequest::default();
//         let database_id = "c5fe9839463a447f9f305d0d1d510f3f".to_string();
//         let mut query = QueryFilter::new();
//         query.query_with_timestamp();
//         let builder = query_db(&request, database_id, query.to_json().unwrap()).await?;
//         if let Ok(status) = get_and_print_reponse(builder).await {
//             assert_eq!(status, 200);
//         }
//         Ok(())
//     }

//     #[tokio::test]
//     async fn init_db_test() -> Result<()> {
//         let request: NotionRequest = NotionRequest::default();
//         let parent_id = "044ce74a839e4c7e8beeea8f49ec2fae".to_string();
//         let schema = json!(
//             {
//                 "Name": {
//                     "title": {}
//                 },
//                 "Description": {
//                     "rich_text": {}
//                 },
//                 "In stock": {
//                     "checkbox": {}
//                 },
//                 "Food group": {
//                     "select": {
//                         "options": [
//                             {
//                                 "name": "🥦Vegetable",
//                                 "color": "green"
//                             },
//                             {
//                                 "name": "🍎Fruit",
//                                 "color": "red"
//                             },
//                             {
//                                 "name": "💪Protein",
//                                 "color": "yellow"
//                             }
//                         ]
//                     }
//                 },
//                 "Price": {
//                     "number": {
//                         "format": "dollar"
//                     }
//                 },
//                 "Last ordered": {
//                     "date": {}
//                 },
//                 "Store availability": {
//                     "type": "multi_select",
//                     "multi_select": {
//                         "options": [
//                             {
//                                 "name": "Duc Loi Market",
//                                 "color": "blue"
//                             },
//                             {
//                                 "name": "Rainbow Grocery",
//                                 "color": "gray"
//                             },
//                             {
//                                 "name": "Nijiya Market",
//                                 "color": "purple"
//                             },
//                             {
//                                 "name": "Gus'\''s Community Market",
//                                 "color": "yellow"
//                             }
//                         ]
//                     }
//                 },
//                 "+1": {
//                     "people": {}
//                 },
//                 "Photo": {
//                     "files": {}
//                 }
//             }
//         );
//         let payload = NotionPayload::new()
//         .parent(ParentType::Page, parent_id.as_str())
//         .icon("🍎".to_string())
//         .title("api testing db".to_string())
//         .cover("https://cdna.artstation.com/p/assets/images/images/010/039/240/large/liu-x-160.jpg?1522232709")
//         .properties(schema);
//         let builder = init_db(&request, "".to_string(), payload.to_json().unwrap()).await?;
//         if let Ok(status) = get_and_print_reponse(builder).await {
//             assert_eq!(status, 200);
//         }
//         Ok(())
//     }

//     ///If parent.type is "page_id" or "workspace", then the only valid key is title. #https://developers.notion.com/reference/page#property-value-object
//     #[tokio::test]
//     async fn init_page_test() -> Result<()> {
//         let request: NotionRequest = NotionRequest::default();
//         let parent_id = "044ce74a839e4c7e8beeea8f49ec2fae".to_string();
//         let schema = json!({
//             "title": [
//                 {
//                     "text": {
//                         "content": "Test api Page"
//                     }
//                 }
//             ]
//         });
//         let payload = NotionPayload::new()
//         .parent(ParentType::Page, parent_id.as_str())
//         .icon("🌹".to_string())
//         .cover("https://cdna.artstation.com/p/assets/images/images/010/039/240/large/liu-x-160.jpg?1522232709")
//         .properties(schema);
//         let builder = init_page(&request, "".to_string(), payload.to_json().unwrap()).await?;
//         if let Ok(status) = get_and_print_reponse(builder).await {
//             assert_eq!(status, 200);
//         }
//         Ok(())
//     }

//     #[tokio::test]
//     async fn get_user_info_test() -> Result<()> {
//         let request: NotionRequest = NotionRequest::default();
//         let builder = get_user_info(&request).await?;
//         if let Ok(status) = get_and_print_reponse(builder).await {
//             assert_eq!(status, 200);
//         }
//         Ok(())
//     }
// }
