use notify_rust::Notification;
use tokio::task;
use log::error;

pub async fn show_notification() -> bool {
    task::spawn_blocking(move || {
        let mut response = false;
        match Notification::new()
            .summary("Job Scheduler")
            .body("Do you want to proceed with the scheduled job?")
            .icon("dialog-question")
            .action("yes", "Yes")
            .action("no", "No")
            .show()
        {
            Ok(notification) => {
                notification.wait_for_action(|action| {
                    match action {
                        "yes" => {
                            response = true;
                        },
                        _ => {
                            response = false;
                        }
                    };
                });
                response
            },
            Err(e) => {
                error!("Failed to show notification: {}", e);
                false
            }
        }
    }).await.unwrap_or_else(|_| {
        error!("Notification task failed");
            false
    })
}
