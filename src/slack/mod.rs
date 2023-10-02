use crate::prelude::*;
use reqwest::{header::HeaderMap, Client};

pub struct SlackApi {
    client: Client,
    headers: HeaderMap,
    base_url: String,
}

impl SlackApi {
    fn new() -> Self {
        
    }
}