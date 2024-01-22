use dotenv::dotenv;
mod cli;
mod commands;
mod notion;
mod slack;

mod prelude {
    pub use std::env;
    pub use crate::cli::*;
    pub use crate::notion::*;
    pub use crate::slack::*;
    pub use clap::{Parser, Subcommand, Args};
    pub use colored::Colorize;
    pub use crate::commands::*;
    pub use anyhow::Error;
    pub use serde::{Deserialize, Serialize};
    pub use async_trait::async_trait;
    pub use csv::{ReaderBuilder, WriterBuilder};
}

use prelude::*;

#[tokio::main]
async fn main() {
    dotenv().ok();
    Cli::parse().run().await;
}
