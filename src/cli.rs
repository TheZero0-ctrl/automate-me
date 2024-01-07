use anyhow::Error;

use crate::prelude::*;
use std::process::ExitCode;

#[async_trait]
pub trait RunCommand{
    async fn run(self) -> Result<(), Error>;
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands  {
    /// get random article to read from reading list of notion
    GiveMeArticle(give_me_article::GiveMeArticle),
    /// generate stand up and post on slack and sheet based on flag provided
    GenerateStandUp(generate_stand_up::GenerateStandUp)
}

impl Cli {
    pub async fn run(self) -> ExitCode {
        let output = match self.command {
            Commands::GiveMeArticle(give_me_article) => give_me_article.run().await,
            Commands::GenerateStandUp(generate_stand_up) => generate_stand_up.run().await
        };

        match output {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("{}", e);
                ExitCode::FAILURE
            }
        }
    }
}