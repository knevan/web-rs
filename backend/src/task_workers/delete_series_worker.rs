use crate::database::storage::StorageClient;
use crate::database::{DatabaseService, Series};
use anyhow::Context;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_retry2::Retry;
use tokio_retry2::strategy::{FixedInterval, jitter};

#[derive(Debug)]
pub struct DeletionJob {
    series: Series,
}

// Scheduler to pool database for deletion jobs
pub async fn run_deletion_scheduler(
    db_service: DatabaseService,
    job_sender: mpsc::Sender<DeletionJob>,
) {
    println!("[WORKER] Deletion worker started");

    // Interval pooling
    let mut interval = tokio::time::interval(Duration::from_secs(180));
    // Skip frist tick
    interval.tick().await;

    loop {
        interval.tick().await;

        // Find looking until the queue is empty in the DB
        loop {
            match db_service.find_and_lock_series_for_job_deletion().await {
                Ok(Some(series)) => {
                    println!(
                        "[DELETION-WORKER] Found job for series {}, send to worker",
                        series.id
                    );
                    let job = DeletionJob { series };
                    if job_sender.send(job).await.is_err() {
                        eprintln!(
                            "[DELETION-WORKER] CRITICAL: Receiver channel closed. Shutting down."
                        );
                        break;
                    }
                }
                Ok(None) => {
                    println!("[DELETION-WORKER] No jobs found. Sleeping.");
                    break;
                }
                Err(e) => {
                    eprintln!(
                        "[DELETION-WORKER] Error finding job: {}. Retrying later.",
                        e
                    );
                    break;
                }
            }
        }
    }
}

pub async fn run_deletion_worker(
    worker_id: usize,
    db_service: DatabaseService,
    storage_client: Arc<StorageClient>,
    // Use async-channel in the future for more than 1 worker
    mut job_receiver: mpsc::Receiver<DeletionJob>,
) {
    println!("[DELETION-WORKER] Deletion worker {} started", worker_id);

    while let Some(job) = job_receiver.recv().await {
        println!(
            "[DELETION-WORKER] Processing job for series {}",
            job.series.id
        );

        let series_id = job.series.id;
        let retry_strategy =
            FixedInterval::from_millis(1000).map(jitter).take(5);

        let db_clone = db_service.clone();
        let storage_clone = storage_client.clone();

        let result = Retry::spawn(retry_strategy, move || {
            let db_attempt = db_clone.clone();
            let storage_attempt = storage_clone.clone();

            async move {
                execute_full_deletion(series_id, &db_attempt, storage_attempt)
                    .await
                    .map_err(|e| {
                        eprintln!(
                            "[WORKER] Attempt for series {} failed: {}. Retrying again.",
                            series_id, e
                        );
                        tokio_retry2::RetryError::transient(e)
                    })
            }
        }).await;

        if let Err(e) = result {
            eprintln!(
                "[DELETION_WORKER] Job for series {} failed after all retry attempts: {}. Moving to 'deletion_failed'",
                series_id, e
            );

            if let Err(e_update) = db_service
                .update_series_processing_status(series_id, "deletion_failed")
                .await
            {
                eprintln!(
                    "[DELETION-WORKER] CRITICAL: Failed to mark series {} as 'deletion_failed'. Error: {}",
                    series_id, e_update
                );
            }
        } else {
            println!(
                "[DELETION-WORKER] Successfully deleted series {} after retries.",
                series_id
            );
        }
    }

    println!(
        "[DELETION-WORKER] Channel closed {}. Shutting down.",
        worker_id
    );
}

// Execute the full deletion process
async fn execute_full_deletion(
    series_id: i32,
    db_service: &DatabaseService,
    storage_client: Arc<StorageClient>,
) -> anyhow::Result<()> {
    // Get all image keys
    let image_keys = db_service
        .get_image_keys_for_series_deletion(series_id)
        .await
        .context("Failed to get image keys")
        // If no series found, assume no images
        .unwrap_or_default();

    let keys_to_delete: Vec<String> = image_keys
        .iter()
        .flat_map(|keys| keys.all_urls())
        .filter_map(|url| storage_client.extract_object_key_from_url(url))
        .collect();

    if !keys_to_delete.is_empty() {
        storage_client
            .delete_image_objects(&keys_to_delete)
            .await
            .context("Failed to delete image objects")?;
    }

    db_service
        .delete_series_by_id(series_id)
        .await
        .context("Failed to delete series")?;

    println!(
        "[WORKER] Successfully processed and deleted series {}",
        series_id
    );

    Ok(())
}
