use crate::prelude::*;
use rand::{distributions::WeightedIndex, prelude::Distribution, thread_rng};
use std::fs;
use std::io::BufWriter;
use std::fs::File;


#[derive(Deserialize, Debug)]
pub struct APIResponse {
    pub results: Vec<Article>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Article {
    pub id: String,
    #[serde(rename = "url")]
    pub url: String,
    pub properties: Properties,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Properties {
    #[serde(rename = "Did I read it")]
    pub reading_info: ReadingInfo,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ReadingInfo {
    #[serde(rename = "checkbox")]
    pub read_it: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ReadingList {
    pub id: String,
    pub url: String,
    pub did_i_read_it: bool,
    pub priority: i32,
}

pub fn update_reading_list(list_of_articles: &Vec<Article>) -> Result<(), Error> {
    let file_path = env::var("READING_LIST_CSV").unwrap();
    let temp_file_path = "temp_reading_list.csv";

    let mut rdr = ReaderBuilder::new().from_path(&file_path)?;
    let mut wtr = WriterBuilder::new().from_writer(BufWriter::new(File::create(&temp_file_path)?));

    let mut existing_data: Vec<ReadingList> = rdr.deserialize().map(|r| r.unwrap()).collect();
    let existing_ids: Vec<String> = existing_data.iter().map(|r| r.id.clone()).collect();

    for article in list_of_articles {
        let id = article.id.clone();
        let url = article
            .url
            .clone();
        let priority = if article.properties.reading_info.read_it {
            50
        } else {
            100
        };
        if !existing_ids.contains(&article.id) {
            wtr.serialize(ReadingList {
                id,
                url,
                did_i_read_it: article.properties.reading_info.read_it,
                priority,
            })?;
        } else {
            let data_to_update = existing_data.iter_mut().find(|item| item.id == article.id).unwrap();
            data_to_update.did_i_read_it = article.properties.reading_info.read_it;
            wtr.serialize(data_to_update)?;
        }
    }

    wtr.flush()?;

    fs::remove_file(&file_path)?;
    fs::rename(&temp_file_path, &file_path)?;

    Ok(())
}

pub fn randomly_choose_article() -> Result<String, Error> {
    println!("{}", "Choosing article".yellow());
    let file_path = env::var("READING_LIST_CSV").unwrap();
    let temp_file_path = "temp_reading_list.csv";

    let mut rdr = ReaderBuilder::new().from_path(&file_path)?;
    let mut wtr = WriterBuilder::new().from_writer(BufWriter::new(File::create(&temp_file_path)?));

    let mut reading_lists: Vec<ReadingList> = rdr.deserialize().map(|r| r.unwrap()).collect();
    let priorities: Vec<i32> = reading_lists.iter().map(|r| r.priority).collect();

    let dist = WeightedIndex::new(&priorities).unwrap();
    let mut rng = thread_rng();
    let chosen_index = dist.sample(&mut rng);
    let chosen_id = reading_lists[chosen_index].id.clone();
    let mut chosen_url = String::new();

    for record in reading_lists.iter_mut() {
        if chosen_id == record.id {
            record.priority -= 1;
            chosen_url = record.url.clone();
        }
        wtr.serialize(record)?;
    }

    wtr.flush()?;

    fs::remove_file(&file_path)?;
    fs::rename(&temp_file_path, &file_path)?;

    Ok(chosen_url)
}
