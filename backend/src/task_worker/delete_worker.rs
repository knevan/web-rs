use crate::database::DatabaseService;
use crate::database::storage::StorageClient;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_retry2::Retry;
use tokio_retry2::strategy::{FixedInterval, jitter};

pub async fn run_deletion_background_worker(
    mut rx: mpsc::Receiver<i32>,
    db_service: DatabaseService,
    storage_client: StorageClient,
) {
    println!("[WORKER] Deletion worker started");
    while let Some(series_id) = rx.recv().await {
        println!("[WORKER] Deleting series with ID: {}", series_id);

        let db_clone = db_service.clone();
        let storage_clone = storage_client.clone();
        tokio::spawn(async move {
            delete_series_with_retry(series_id, &db_clone, &storage_clone)
                .await;
        });
    }
    println!("[WORKER] Deletion worker channel closed. Shutting down.");
}

// Helper function to extract storage object key from CDN URL
fn extract_object_key_from_url(url: &str, base_url: &str) -> Option<String> {
    // Ensure base url does not have trailing slash
    let trimmed_base_url = base_url.trim_end_matches('/');

    url.strip_prefix(trimmed_base_url)
        // Remove base url(cdn url) and leading slash to get object key
        .map(|remaining| remaining.trim_start_matches('/').to_string())
}

async fn delete_series_with_retry(
    series_id: i32,
    db_service: &DatabaseService,
    storage_client: &StorageClient,
) {
    // Get all image url
    let image_data = match db_service
        .get_image_keys_for_series_deletion(series_id)
        .await
    {
        Ok(Some(data)) => data,
        Ok(None) => {
            println!(
                "[WORKER] Series {} not found for deletion. Assuming already deleted.",
                series_id
            );
            return;
        }
        Err(e) => {
            eprintln!(
                "[WORKER] Failed to get image keys for series {}: {}. Aborting deletion.",
                series_id, e
            );
            return;
        }
    };

    // Prepare key list for deletion from storage
    let keys_to_delete_in_storage: Arc<Vec<String>> = Arc::new({
        let mut keys: Vec<String> = Vec::new();
        let domain_cdn_url = storage_client.domain_cdn_url();

        if let Some(cover_url) = image_data.cover_image_url {
            if let Some(key) =
                extract_object_key_from_url(&cover_url, domain_cdn_url)
            {
                keys.push(key);
            }
        }
        for url in image_data.chapter_image_urls {
            if let Some(key) = extract_object_key_from_url(&url, domain_cdn_url)
            {
                keys.push(key);
            }
        }
        keys
    });

    if keys_to_delete_in_storage.is_empty() {
        println!(
            "[WORKER] No R2 objects to delete for series {}. Proceeding to DB deletion.",
            series_id
        );
    } else {
        // Retry 5 times with exponential backoff
        let retry_strategy =
            FixedInterval::from_millis(1000).map(jitter).take(5);

        let storage_delete_result = Retry::spawn(retry_strategy, || {
            let storage_client_clone = storage_client.clone();
            let keys_clone = Arc::clone(&keys_to_delete_in_storage);

            async move {
                println!("[WORKER] Attempting to delete {} objects from R2 for series {}", keys_clone.len(), series_id);

            storage_client_clone.delete_image_objects(&keys_clone)
                .await
                .map_err(tokio_retry2::RetryError::transient)
        }
    })
    .await;

        if storage_delete_result.is_err() {
            eprintln!(
                "[WORKER] CRITICAL: Failed to delete R2 objects for series {} after multiple retries. Halting process. MANUAL INTERVENTION REQUIRED.",
                series_id
            );
            // Hentikan proses jika R2 gagal setelah semua retry.
            return;
        }
    }

    // If deletion from R2 is successful, delete from DB
    println!(
        "[WORKER] R2 objects for series {} deleted (or confirmed absent). Proceeding to DB deletion.",
        series_id
    );
    if let Err(e) = db_service.delete_series_by_id(series_id).await {
        eprintln!(
            "[WORKER] CRITICAL: R2 delete succeeded but DB delete failed for series {}: {}. MANUAL INTERVENTION REQUIRED.",
            series_id, e
        );
    } else {
        println!(
            "[WORKER] Series {} fully deleted from DB. Deletion complete.",
            series_id
        );
    }
}
