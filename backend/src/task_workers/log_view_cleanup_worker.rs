use tokio_cron_scheduler::{Job, JobScheduler};

use crate::database::DatabaseService;

pub async fn run_log_view_cleanup_worker(db_service: DatabaseService) {
    let log_cleanup = async {
        let scheduler = JobScheduler::new().await?;
        let db_clone = db_service.clone();
        let cron_exp = "0 0 2 * * * *";

        let cleanup_job = Job::new_async(cron_exp, move |_uuid, _locked| {
            let db = db_clone.clone();
            Box::pin(async move {
                println!("[CRON] Starting daily view log cleanup (Pruning > 35 days)...");

                match db.cleanup_old_view_logs().await {
                    Ok(deleted) => {
                        if deleted > 0 {
                            println!("[CRON] Cleanup success. Pruned {} old rows.", deleted);
                        } else {
                            println!("[CRON] Cleanup ran. Database is clean (no rows > 40 days).");
                        }
                    }
                    Err(e) => eprintln!("[CRON] Cleanup failed: {}", e),
                }
            })
        })?;

        scheduler.add(cleanup_job).await?;
        scheduler.start().await?;

        Ok::<JobScheduler, anyhow::Error>(scheduler)
    }
    .await;

    match log_cleanup {
        Ok(_scheduler) => {
            // CRITICAL: Hold this task so that the scheduler doesn't get dropped.
            // The variable '_scheduler' is owned by this scope. While pending() waits,
            // the scheduler remains alive and cron jobs continue to run in the background.
            std::future::pending::<()>().await;
        }
        Err(e) => {
            eprintln!(
                "[FATAL LOG VIEW CLEANUP WORKER ERROR] Log View Cleanup Scheduler died: {}",
                e
            );
        }
    }
}
