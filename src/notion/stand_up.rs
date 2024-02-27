use crate::prelude::*;
use chrono::prelude::*;
use std::collections::HashMap;
use std::str::FromStr;
use std::fs::File;
use std::io::Read;

#[derive(Serialize, Deserialize, Debug)]
pub struct APIResponse {
    pub results: Vec<Task>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Filter {
    pub filter: FilterDetails,
    pub sorts: Vec<Sort>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FilterDetails {
    pub and: Vec<FilterCondition>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Sort {
    pub property: String,
    pub direction: String,
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

impl FromStr for Status {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "in progress" => Ok(Status::InProgress),
            "done" => Ok(Status::Done),
            "to do" => Ok(Status::ToDo),
            _ => Err(()),
        }
    }
}

impl ToString for Status {
    fn to_string(&self) -> String {
        match self {
            Status::InProgress => String::from("In progress"),
            Status::Done => String::from("Done"),
            Status::ToDo => String::from("To Do"),
        }
    }
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
    #[serde(rename = "Projects")]
    project: Project
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StatusInfo {
    pub status: TaskStatus
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TaskStatus {
    pub name: Status
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Name {
    pub title: Vec<Title>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Title {
    pub plain_text: String,
    pub text: TitleTask
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TitleTask {
    pub content: String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TaskToAdd {
    properties: Properties,
    parent: Parent,
}

#[derive(Debug, Deserialize, Serialize)]
struct Parent {
    database_id: String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Project {
    #[serde(rename = "type")]
    relation_type: String,
    relation: Vec<Relation>,
    has_more: bool
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Relation {
    id: String
}

impl TaskToAdd {
    pub fn new(task: String, status: String, database_id: String, project: String) -> Self {
        let status = Status::from_str(&status).unwrap();
        Self {
            properties: Properties {
                name: Name {
                    title: vec![
                        Title {
                            plain_text: task.clone(),
                            text: TitleTask {
                                content: task
                            }
                        }
                    ]
                },
                status: StatusInfo {
                    status: TaskStatus {
                        name: status
                    }
                },
                project: Project {
                    relation_type: String::from("relation"),
                    relation: vec![
                        Relation {
                            id: project_to_id(&project).unwrap() 
                        }
                    ],
                    has_more: false
                }
            },
            parent: Parent {
                database_id
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct ProjectMapping {
    mapping: HashMap<String, String>,
}

fn read_project_mapping(file_path: &str) -> Result<ProjectMapping, Box<dyn std::error::Error>> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mapping: ProjectMapping = serde_json::from_str(&contents)?;

    Ok(mapping)
}


fn project_to_id(project: &str) -> Option<String> {
    let file_path = env::var("PROJECT_MAPPING_JSON").unwrap();
    if let Ok(project_mapping) = read_project_mapping(&file_path) {
        project_mapping.mapping.get(project.to_lowercase().as_str()).cloned()
    } else {
        None
    }
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
            sorts: vec![
                Sort {
                    property: String::from("Last edited time"),
                    direction: String::from("ascending") 
                }
            ],
        }
    }
}

impl APIResponse {
    fn classify_tasks(&self) -> ClassifiedTasks {
        let mut today = Vec::new();
        let mut tomorrow = Vec::new();
        for task in self.results.iter() {
            let name = task.properties.name.title[0].plain_text.clone();
            let status = &task.properties.status.status.name;

            match status {
                Status::Done => {
                    today.push(name);
                },
                Status::InProgress => {
                    let wip = format!("WIP {}", name);
                    today.push(wip);
                },
                _ => {
                    tomorrow.push(name);
                }
            }

        }

        ClassifiedTasks {
            today,
            tomorrow,
        }
    }

    fn format_tasks(&self, tasks: Vec<String>) -> String {
        tasks
            .iter()
            .map(|task| format!(" • {}", task))
            .collect::<Vec<String>>()
            .join("\n")
    }
    pub fn tasks_for_standup(&self) -> String {
        let classified_tasks = self.classify_tasks();     
        let today = formatted_today();
        let today_tasks_str: String = self.format_tasks(classified_tasks.today);
        let tomorrow_tasks_str: String = self.format_tasks(classified_tasks.tomorrow);
        let result = format!(
            "Stand-up {}\nToday\n{}\nTomorrow\n{}\nBlocker\n • None",
            today, today_tasks_str, tomorrow_tasks_str
        );

        result
    }

    pub fn tasks_for_timelog(&self) -> String {
        self.format_tasks(self.classify_tasks().today)
    }
}

struct ClassifiedTasks {
    today: Vec<String>,
    tomorrow: Vec<String>,
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
