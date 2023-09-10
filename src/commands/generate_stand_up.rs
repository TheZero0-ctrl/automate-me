use crate::prelude::*;

#[derive(Debug, Args)]
pub struct GenerateStandUp {
    /// Flag for sending stand up to slack
    #[arg(short, long)]
    slack: bool,

    /// Flag for updating timelog on google sheet
    #[arg(short, long)]
    timelog: bool,
}

#[async_trait]
impl RunCommand for GenerateStandUp {
    async fn run(self) ->  Result<(), Error> {
        println!("{}", "Generating stand up".yellow());
        let api = NotionApi::new(
            env::var("NOTION_TASK_DATABASE_ID").unwrap(),
            "databases".to_string() 
        );
        let tasks = api.get_tasks_for_standup().await?;
        let stand_up = stand_up::generate_stand_up(tasks).await;
        println!("{}", stand_up.green());
        if self.slack {
            println!("{}", "Sending stand up to slack".yellow());
        }
        Ok(())
    }
}
