pub mod slack_message;

use crate::prelude::*;
use reqwest::{header::HeaderMap, Client};

pub struct SlackApi {
    client: Client,
    headers: HeaderMap,
    base_url: String,
}

impl SlackApi {
    pub fn new() -> Self {
        let api_key = env::var("SLACK_USER_TOKEN").unwrap();
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", api_key).parse().unwrap(),
        );
        headers.insert("Accept", "application/json".parse().unwrap());
        Self {
            client: reqwest::Client::new(),            
            headers,
            base_url: "https://slack.com/api".to_string(),
        }
    }

    pub async fn send_message(&self, message: String, channel: String) -> Result<(), Error> {
        println!("{}", "Sending message to Slack".yellow());

        let response = self.client
        .post(&format!("{}/chat.postMessage", self.base_url))
        .json(&slack_message::StandupMessage::new(channel, message))
        .headers(self.headers.clone())
        .send()
        .await?
        .json::<slack_message::MessageResponse>()
        .await?;

        if response.ok {
            println!("{}", "Message sent to Slack".green());
        } else {
            println!("{}", "Failed to send message to Slack".red());
            println!("{}", response.error.unwrap().red());
        }

        Ok(())
    }
}
