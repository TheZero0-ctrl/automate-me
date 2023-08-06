pub mod reading_list;
use crate::prelude::*;
use reqwest::{header::HeaderMap, Client};

pub struct NotionApi {
    pub client: Client,
    pub headers: HeaderMap,
    pub base_url: String,
    pub database_id: String,
}


impl NotionApi {
    pub fn new(id: String, url: String) -> Self {
        let api_key = env::var("NOTION_API_KEY").unwrap();
        let client = reqwest::Client::new();
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", format!("Bearer {}", api_key).parse().unwrap());
        headers.insert("Notion-Version", "2022-06-28".parse().unwrap());
        headers.insert("Accept", "application/json".parse().unwrap());
        Self {
            client,
            headers,
            base_url: format!("https://api.notion.com/v1/{}/{}/query",url, id),
            database_id: id,
        }
        
    }

    pub async fn get_articles(&self) -> Result<Vec<reading_list::Article>, Error> {
        println!("{}", "Getting articles from Notion API".yellow());

        let response = self.client
        .post(self.base_url.clone())
        .headers(self.headers.clone())
        .send()
        .await?
        .json::<reading_list::APIResponse>()
        .await?;

        reading_list::update_reading_list(&response.results)?;
        Ok(response.results)
    }

    pub async fn get_article(&self) -> Result<String, Error> {
        self.get_articles().await.unwrap();
        reading_list::randomly_choose_article()
        // Ok("hello".to_string())
    }
}