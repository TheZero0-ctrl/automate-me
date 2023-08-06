use crate::prelude::*;

#[derive(Debug, Args)]
pub struct GiveMeArticle {}

#[async_trait]
impl RunCommand for GiveMeArticle {
    async fn run(self) -> Result<(), Error> {
        let api = NotionApi::new(
            env::var("NOTION_READING_LIST_DATABASE_ID").unwrap(),
            "databases".to_string() 
        );
        let url = api.get_article().await?;
        open::that(&url)?;
        println!("{}    {}","Your article is".green(), url.blue());
        Ok(())
    }
}