use crate::prelude::*;
use chrono::prelude::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct APIResponse {
    pub results: Vec<Task>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Filter {
    pub filter: FilterDetails,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FilterDetails {
    pub and: Vec<FilterCondition>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum FilterCondition {
    #[serde(rename = "or")]
    Or(Vec<StatusCondition>),
    #[serde(untagged)]
    LastEditedTime(LastEditedTimeCondition),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StatusCondition {
    pub property: String,
    pub status: StatusEquals,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct  StatusEquals {
    pub equals: Status
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Status {
    #[serde(rename = "In progress")]
    InProgress,
    Done,
    #[serde(rename = "To Do")]
    ToDo,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LastEditedTimeCondition {
    pub property: String,
    pub last_edited_time: OnOrAfter,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OnOrAfter {
    #[serde(rename = "on_or_after")]
    pub date: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Task {
    pub properties: Properties
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Properties {
    #[serde(rename = "Name")]
    pub name: Name,
    #[serde(rename = "Status")]
    pub status: StatusInfo,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StatusInfo {
    pub status: TaskStatus
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TaskStatus {
    name: Status
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Name {
    pub title: Vec<Title>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Title {
    pub plain_text: String
}

impl Filter {
    pub fn new() -> Self {
        Self {
            filter: FilterDetails {
                and: vec![
                    FilterCondition::Or(vec![
                        StatusCondition {
                            property: String::from("Status"),
                            status: StatusEquals {
                                equals: Status::InProgress,
                            }
                        },
                        StatusCondition {
                            property: String::from("Status"),
                            status: StatusEquals {
                                equals: Status::Done,
                            }
                        },
                        StatusCondition {
                            property: String::from("Status"),
                            status: StatusEquals {
                                equals: Status::ToDo,
                            }
                        },
                    ]),
                    FilterCondition::LastEditedTime(LastEditedTimeCondition {
                        property: String::from("Last edited time"),
                        last_edited_time: OnOrAfter {
                            date: Local::now().format("%Y-%m-%d").to_string(),
                        },
                    }),
                ],
            },
        }
    }
}

pub async fn generate_stand_up(tasks: Vec<Task>) -> String {
    let mut today_section: Vec<String>  = Vec::new();
    let mut tomorrow_section: Vec<String> = Vec::new();
    let today = formatted_today();
    for task in tasks.iter() {
        let name = task.properties.name.title[0].plain_text.clone();
        let status = &task.properties.status.status.name;

        match status {
            Status::Done => {
                today_section.push(name);
            },
            Status::InProgress => {
                let wip = format!("WIP {}", name);
                today_section.push(wip);
            },
            _ => {
                tomorrow_section.push(name);
            }
        }
    }

    let today_tasks_str: String = today_section
    .iter()
    .map(|task| format!(" • {}", task))
    .collect::<Vec<String>>()
    .join("\n");

    let tomorrow_tasks_str: String = tomorrow_section
        .iter()
        .map(|task| format!(" • {}", task))
        .collect::<Vec<String>>()
        .join("\n");

    let result = format!(
        "Stand-up {}\nToday\n{}\nTomorrow\n{}\nBlocker\n • None",
        today, today_tasks_str, tomorrow_tasks_str
    );

    result
}

fn formatted_today() -> String {
    let today = Local::now();
    let day = today.day();

    let day_suffix = match day {
        1 | 21 | 31 => "st",
        2 | 22 => "nd",
        3 | 23 => "rd",
        _ => "th",
    };

    format!("{}{}{}", today.format("%b"), today.format(" %d"), day_suffix)
}