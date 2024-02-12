use crate::prelude::*;

#[derive(Debug, Args)]
pub struct GiveMeArticle {}

#[async_trait]
impl RunCommand for GiveMeArticle {
    async fn run(self) -> Result<(), Error> {
        let database_id = env::var("NOTION_READING_LIST_DATABASE_ID").unwrap();
        let api = NotionApi::new(
            &format!(
                "databases/{}/query",
                database_id
            )
        );
        let url = api.get_article().await?;
        open::that(&url)?;
        println!("{}    {}","Your article is".green(), url.blue());
        Ok(())
    }
}
