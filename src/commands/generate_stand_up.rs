use crate::prelude::*;

#[derive(Debug, Args)]
pub struct GenerateStandUp {
    /// Flag for sending stand up to slack
    #[arg(short, long)]
    slack: bool,

    /// Flag for updating timelog on google sheet
    #[arg(short, long)]
    timelog: bool,
    
    /// data to fill out in timelog In Office header
    #[arg(short, long, default_value = "WFH")]
    in_office: String,

    /// data to fill out in timelog Hours header
    #[arg(short = 'w', long, default_value = "8")]
    hours: String,
}

#[async_trait]
impl RunCommand for GenerateStandUp {
    async fn run(self) ->  Result<(), Error> {
        println!("{}", "Generating stand up".yellow());
        let api = NotionApi::new(
            env::var("NOTION_TASK_DATABASE_ID").unwrap(),
            "databases".to_string() 
        );
        let tasks = api.get_tasks().await?;
        let stand_up = tasks.tasks_for_standup();
        println!("{}", stand_up.green());
        if self.slack {
            let slack_api = SlackApi::new();
            slack_api
                .send_message(stand_up.clone(), env::var("SLACK_CHANNEL")
                .unwrap())
                .await?;
        }

        if self.timelog {
            let sheet_api = GoogleSheetsApi::new(
                env::var("SHEET_ID").unwrap(),
            ).await;
            sheet_api.post_timelog(
                tasks.tasks_for_timelog(),
                self.in_office,
                self.hours,
            ).await?;
        }
        Ok(())
    }
}
