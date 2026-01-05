use tokio_cron_scheduler::{Job, JobScheduler};

use crate::database::DatabaseService;

pub async fn run_cleanup_password_reset_token_worker(db_service: DatabaseService) {
    let token_cleanup = async {
        let scheduler = JobScheduler::new().await?;
        let db_clone = db_service.clone();
        let cron_exp = "0 0 0,12 * * * *";

        let cleanup_job = Job::new_async(cron_exp, move |_uuid, _locked| {
            let db = db_clone.clone();

            Box::pin(async move {
                println!("[CRON] Starting password reset token cleanup...");

                match db.cleanup_password_reset_token().await {
                    Ok(delete_token) => {
                        if delete_token > 0 {
                            println!("Deleting password reset token: {}", delete_token);
                        } else {
                            println!("[CRON] No expired tokens found.");
                        }
                    }
                    Err(error) => eprintln!("Failed to cleanup password reset token: {}", error),
                }
            })
        })?;

        scheduler.add(cleanup_job).await?;
        scheduler.start().await?;

        Ok::<JobScheduler, anyhow::Error>(scheduler)
    }
    .await;

    match token_cleanup {
        Ok(_scheduler) => {
            // CRITICAL: Hold this task so that the scheduler doesn't get dropped.
            // The variable '_scheduler' is owned by this scope. While pending() waits,
            // the scheduler remains alive and cron jobs continue to run in the background.
            std::future::pending::<()>().await;
        }
        Err(e) => {
            eprintln!(
                "[FATAL DELETE TOKEN PASSWORD ERROR] Reset Password Token Cleanup Scheduler died: {}",
                e
            );
        }
    }
}
