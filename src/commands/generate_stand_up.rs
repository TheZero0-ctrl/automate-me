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

impl GenerateStandUp {
    pub fn init() -> Self {
        GenerateStandUp {
            slack: false,
            timelog: false,
            in_office: String::from("WFH"),
            hours: String::from("8"),
        }
    }

    pub fn init_with_slack() -> Self {
        GenerateStandUp {
            slack: true,
            timelog: false,
            in_office: String::from("WFH"),
            hours: String::from("8"),
        }
    }

    pub fn init_with_timelog() -> Self {
        GenerateStandUp {
            slack: false,
            timelog: true,
            in_office: String::from("WFH"),
            hours: String::from("8"),
        }
    }

    pub async fn stand_up(&self) -> Result<stand_up::APIResponse, Error> {
        let database_id = env::var("NOTION_TASK_DATABASE_ID").unwrap();
        let api = NotionApi::new(
            &format!(
                "databases/{}/query",
                database_id
            )
        );

        let stand_up = api.get_tasks().await?;

        Ok(stand_up)
    }

    pub async fn send_to_slack(&self, message: String) -> Result<(), Error> {
        let slack_api = SlackApi::new();
        slack_api.send_message(message, env::var("SLACK_CHANNEL").unwrap()).await
    }

    pub async fn log_timelog(&self, message: String) -> Result<(), Error> {
        let sheet_api = GoogleSheetsApi::new(
            env::var("SHEET_ID").unwrap(),
        ).await;

        sheet_api.post_timelog(
            message,
            self.in_office.clone(),
            self.hours.clone(),
        ).await
    }
}

#[async_trait]
impl RunCommand for GenerateStandUp {
    async fn run(self) ->  Result<(), Error> {
        println!("{}", "Generating stand up".yellow());
        let tasks = self.stand_up().await?;
        let stand_up = tasks.tasks_for_standup();
        println!("{}", stand_up.green());

        if self.slack {
            self.send_to_slack(stand_up.clone()).await?;
        }

        if self.timelog {
            let task_for_timelog = tasks.tasks_for_timelog();
            self.log_timelog(task_for_timelog).await?;
        }
        Ok(())
    }
}
