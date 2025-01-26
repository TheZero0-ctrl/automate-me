use crate::prelude::*;
use tokio_cron_scheduler::{JobScheduler, Job};
use log::info;
use simplelog::{CombinedLogger, WriteLogger, LevelFilter, ConfigBuilder};
use crate::utils;

#[derive(Debug, Args)]
pub struct RunDaemon;

#[async_trait]
impl RunCommand for RunDaemon {
    async fn run(self) -> Result<(), Error> {
        let log_file = "/tmp/rust-app.log";
        let log_config = ConfigBuilder::new()
            .set_time_level(LevelFilter::Error)
            .set_location_level(LevelFilter::Off)
            .set_target_level(LevelFilter::Off)
            .set_thread_level(LevelFilter::Off)
            .set_time_format_rfc3339()
            .build();

        CombinedLogger::init(vec![
            WriteLogger::new(
                LevelFilter::Info,
                log_config,
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(log_file)
                    .expect("Can't open log file"),
            ),
        ]).expect("Failed to initialize logger");

        // Initialize scheduler after daemonization
        let sched = JobScheduler::new().await?;

        sched.add(
            Job::new_async("*/60 * * * * *", |_uuid, _l| {
                Box::pin(async {
                    info!("Scheduled task executed");
                    let proceed = utils::show_notification().await;
                    info!("Notification response: {}", proceed);
                    if proceed {
                        info!("Proceeding with scheduled task");
                        let generate_stand_up = generate_stand_up::GenerateStandUp::init();
                        let response = generate_stand_up.stand_up().await.unwrap();
                        let tasks = response.results;

                        let got_today_tasks: bool = tasks.iter().any(|task| {
                            match task.properties.status.status.name {
                                stand_up::Status::Done => true,
                                _ => false,
                            }
                        });

                        let got_tomorrow_tasks: bool = tasks.iter().any(|task| {
                            match task.properties.status.status.name {
                                stand_up::Status::InProgress => true,
                                _ => false,
                            }
                        });

                        if got_today_tasks {
                            info!("Got today tasks");
                        } else {
                            info!("No today tasks");
                        }

                        if got_tomorrow_tasks {
                            info!("Got tomorrow tasks");
                        } else {
                            info!("No tomorrow tasks");
                        }
                    }
                })
            })?
        ).await?;

        sched.start().await?;

        tokio::signal::ctrl_c().await?;
        Ok(())
    }
}

