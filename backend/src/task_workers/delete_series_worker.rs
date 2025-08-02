use crate::database::DatabaseService;
use crate::database::storage::StorageClient;
use anyhow::Context;
use std::sync::Arc;
use std::time::Duration;
use tokio_retry2::Retry;
use tokio_retry2::strategy::{FixedInterval, jitter};

pub async fn run_deletion_background_worker(
    db_service: DatabaseService,
    storage_client: Arc<StorageClient>,
) {
    println!("[WORKER] Deletion worker started");

    loop {
        let job_processed =
            process_one_available_job(&db_service, storage_client.clone())
                .await;

        // Empty queue wait longer, if job available wait shortlyx`
        if !job_processed {
            tokio::time::sleep(Duration::from_secs(60)).await;
        } else {
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }
}

async fn process_one_available_job(
    db_service: &DatabaseService,
    storage_client: Arc<StorageClient>,
) -> bool {
    let series_result =
        db_service.find_and_lock_series_for_job_deletion().await;

    let series = match series_result {
        Ok(Some(s)) => s,
        Ok(None) => return false,
        Err(e) => {
            eprintln!("[WORKER] Error finding job: {}. Retrying later.", e);
            return false;
        }
    };

    let retry_strategy = FixedInterval::from_millis(1000).map(jitter).take(5);

    let db_service_owned = db_service.clone();

    let result = Retry::spawn(retry_strategy, move || {
        let db_service_attempt = db_service_owned.clone();
        let storage_client_attempt = storage_client.clone();

        async move {
            execute_full_deletion(
                series.id,
                &db_service_attempt,
                storage_client_attempt,
            )
            .await
            .map_err(|e| {
                eprintln!(
                    "[WORKER] Attempt for series {} failed: {}. Retrying again.",
                    series.id, e
                );
                tokio_retry2::RetryError::transient(e)
            })
        }
    })
    .await;

    // Handle the result after all try attempts
    if result.is_err() {
        eprintln!(
            "[WORKER] Job for series {} failed after 5 attempt. Moving to 'deletion_failed'.",
            series.id
        );

        if let Err(e) = db_service
            .update_series_processing_status(series.id, "deletion_failed")
            .await
        {
            eprintln!(
                "[WORKER] CRITICAL: Failed to mark series {} as 'deletion_failed'. Error: {}",
                series.id, e
            );
        }
    }

    true
}

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
