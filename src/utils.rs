// #[macro_use]
// pub mod macros {
//     #[macro_export]
//     macro_rules! format_url {
//         ($r: ident, $e:ident) => {
//             $r.replace("{}", $e.as_str())
//         };
//     }
// }

pub fn format_url(origin: &str, params: impl AsRef<str>) -> String {
    origin.replace("{}", params.as_ref())
}

#[cfg(test)]
pub mod helper {
    use anyhow::Result;
    use log::info;
    use reqwest::{RequestBuilder, StatusCode};

    pub async fn get_and_print_reponse(builder: RequestBuilder) -> Result<StatusCode> {
        let response = builder.send().await?;
        let status = response.status();
        let body = &response.text().await?;
        // let response: Value = serde_json::from_str(body)?;
        info!("res body: {}", body);
        Ok(status)
    }
}
