use dotenv::dotenv;
mod cli;
mod commands;
mod notion;
mod slack;
mod sheet;
mod utils;
use daemonize::Daemonize;
use std::fs::File;
use std::process::ExitCode;

mod prelude {
    pub use std::env;
    pub use crate::cli::*;
    pub use crate::notion::*;
    pub use crate::slack::*;
    pub use crate::sheet::*;
    pub use clap::{Parser, Subcommand, Args};
    pub use colored::Colorize;
    pub use crate::commands::*;
    pub use anyhow::Error;
    pub use serde::{Deserialize, Serialize};
    pub use async_trait::async_trait;
    pub use csv::{ReaderBuilder, WriterBuilder};
}

use prelude::*;

// #[tokio::main]
fn main() -> ExitCode {
    dotenv().ok();
    let cli = Cli::parse();
    if matches!(cli.command, Commands::RunDaemon(_)) {
        let daemonize = Daemonize::new()
            .pid_file("/tmp/rust-app.pid")
            .stdout(File::create("/tmp/rust-app.out").unwrap())
            .stderr(File::create("/tmp/rust-app.err").unwrap());

        daemonize.start().expect("Daemonization failed");
    }
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(cli.run())
}
