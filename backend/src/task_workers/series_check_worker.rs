use crate::app::orchestrator;
use crate::database::storage::StorageClient;
use crate::database::{DatabaseService, Series, SeriesStatus};
use crate::scraping::model::SitesConfig;
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug)]
pub struct SeriesCheckJob {
    pub series: Series,
}

// Scheduler for pooling DB
pub async fn run_series_check_scheduler(
    db_service: DatabaseService,
    job_sender: async_channel::Sender<SeriesCheckJob>,
) {
    println!("[SERIES-SCHEDULER] Starting...");

    // Interval to check db for job
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    // Skip first tick
    interval.tick().await;

    loop {
        interval.tick().await;

        loop {
            match db_service.find_and_lock_series_for_check().await {
                Ok(Some(series)) => {
                    println!(
                        "[SERIES-SCHEDULER] Found series for check {}, id {}",
                        series.title, series.id
                    );
                    let job = SeriesCheckJob { series };
                    if job_sender.send(job).await.is_err() {
                        eprintln!(
                            "[SERIES-SCHEDULER] CRITICAL: Receiver channel closed. Shutting down."
                        );
                        return;
                    }
                }
                Ok(None) => {
                    // No job found, wait for next tick
                    break;
                }
                Err(e) => {
                    eprintln!(
                        "[SERIES-SCHEDULER] Error finding {}. Retrying later",
                        e
                    );
                    break;
                }
            }
        }
    }
}

pub async fn run_series_check_worker(
    worker_id: usize,
    db_service: DatabaseService,
    storage_client: Arc<StorageClient>,
    http_client: Client,
    sites_config: Arc<SitesConfig>,
    job_receiver: async_channel::Receiver<SeriesCheckJob>,
) {
    println!("[SERIES-WORKER {}] Starting...", worker_id);

    while let Ok(job) = job_receiver.recv().await {
        let series = job.series;
        println!(
            "[SERIES-WORKER] Checking series {}, id {}",
            series.title, series.id
        );

        let result = orchestrator::run_series_check(
            series.clone(),
            http_client.clone(),
            &db_service,
            sites_config.clone(),
            storage_client.clone(),
        )
        .await;

        // After completion (successful or unsuccessful), update the next check schedule.
        let (final_status, next_check_time) = if let Err(e) = result {
            eprintln!(
                "[SERIES-WORKER] Error checking series {}:{}. Retrying later: {}",
                series.title, series.id, e
            );
            // If failed, retry again after 1 hour
            (
                SeriesStatus::Error,
                Some(chrono::Utc::now() + chrono::Duration::hours(1)),
            )
        } else {
            // If successful, let DB calculate the next schedule
            (SeriesStatus::Ongoing, None)
        };

        if let Err(e) = db_service
            .update_series_check_schedule(
                series.id,
                Some(final_status),
                next_check_time,
            )
            .await
        {
            eprintln!(
                "[SERIES-WORKER] CRITICAL: Failed to update schedule for series {}: {}",
                series.id, e
            );
        }
    }
    println!(
        "[SERIES-WORKER {}] Channel closed. Shutting down...",
        worker_id
    );
}
