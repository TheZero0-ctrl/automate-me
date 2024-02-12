use crate::prelude::*;

#[derive(Debug, Args)]
pub struct AddTask {
    /// Title of task
    #[arg(short, long)]
    task: String,

    /// Status of task
    /// Possible values: "to do", "in progress", "done"
    #[arg(short, long, default_value = "done")]
    status: String,

    /// Project of task
    #[arg(short, long)]
    project: String,
}

#[async_trait]
impl RunCommand for AddTask {
    async fn run(self) -> Result<(), Error> {
        println!("{}", "Adding task".yellow());
        let database_id = env::var("NOTION_TASK_DATABASE_ID").unwrap();
        let api = NotionApi::new("pages");

        api.add_task(
            self.task.clone(),
            self.status,
            database_id,
            self.project
        ).await?;
        println!("{}", self.task);
        Ok(())
    }
}
