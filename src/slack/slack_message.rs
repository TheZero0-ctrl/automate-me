use crate::prelude::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct MessageResponse {
    pub ok: bool,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StandupMessage {
    pub blocks: Vec<Block>,
    pub channel: String,
    pub as_user: bool,
    pub mrkdwn: bool,
    pub text: String,
}

impl StandupMessage {
    pub fn new(channel: String, text: String) -> Self {
        let block = Block {
            block_type: "section".to_string(),
            text: Message {
                text: text.clone(),
                message_type: "mrkdwn".to_string()
            }
        };
        Self {
            blocks: vec![block],
            channel,
            as_user: true,
            mrkdwn: true,
            text
        }
    }
    
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Block {
    #[serde(rename = "type")]
    pub block_type: String,
    pub text: Message
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Message {
    pub text: String,
    #[serde(rename = "type")]
    pub message_type: String,
}
