use std::sync::Arc;
use std::time::Duration;

use arc_swap::ArcSwap;
use reqwest::Client;
use tokio::time::MissedTickBehavior;

use crate::database::storage::StorageClient;
use crate::database::{
    ChapterStatus, DatabaseService, DownloadJobData, Series, SeriesCheckTaskInfo,
};
use crate::processing::coordinator::process_single_chapter;
use crate::scraping::model::SitesConfig;
use crate::scraping::parser::ChapterInfo;

#[derive(Debug)]
pub struct SeriesProcessingContext {
    pub series_id: i32,
    pub series_title: String,
    pub source_url: String,
}

impl From<&Series> for SeriesProcessingContext {
    fn from(series: &Series) -> Self {
        Self {
            series_id: series.id,
            series_title: series.title.clone(),
            source_url: series.current_source_url.clone(),
        }
    }
}

impl From<&SeriesCheckTaskInfo> for SeriesProcessingContext {
    fn from(task: &SeriesCheckTaskInfo) -> Self {
        Self {
            series_id: task.id,
            series_title: task.title.clone(),
            source_url: task.current_source_url.clone(),
        }
    }
}

impl From<&DownloadJobData> for SeriesProcessingContext {
    fn from(job: &DownloadJobData) -> Self {
        Self {
            series_id: job.series_id,
            series_title: job.series_title.clone(),
            source_url: job.series_url.clone(),
        }
    }
}

pub async fn run_chapter_download_worker(
    worker_id: usize,
    db_service: DatabaseService,
    storage_client: Arc<StorageClient>,
    http_client: Client,
    sites_config: Arc<ArcSwap<SitesConfig>>,
) {
    println!("[DOWNLOAD-WORKER] Starting... Worker ID {}", worker_id);

    let mut interval = tokio::time::interval(Duration::from_secs(10));
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        interval.tick().await;

        let pending_jobs = match db_service.find_and_lock_pending_chapters(20).await {
            Ok(jobs) => jobs,
            Err(e) => {
                eprintln!(
                    "[DOWNLOAD-WORKER] ID: {}. Failed to find and lock pending chapters: {}",
                    worker_id, e
                );
                continue;
            }
        };

        if pending_jobs.is_empty() {
            continue;
        }

        println!(
            "[DOWNLOAD-SCHEDULER] Dispatching {} chapters to workers...",
            pending_jobs.len()
        );

        for download_job in pending_jobs {
            let storage_client = storage_client.clone();
            let db_service = db_service.clone();
            let http_client = http_client.clone();
            let sites_config = sites_config.clone();

            tokio::spawn(async move {
                let series_ctx: SeriesProcessingContext = (&download_job).into();

                let chapter_info = ChapterInfo {
                    url: download_job.chapter_url.clone(),
                    number: download_job.chapter_number,
                };

                // Get config, if failed skip
                let config_snapshot = sites_config.load();

                let site_config = match config_snapshot.get_site_config(&download_job.source_host) {
                    Some(config) => config,
                    None => {
                        eprintln!(
                            "[DOWNLOAD-WORKER] No config for host: {}",
                            download_job.source_host
                        );

                        // Update status to error if no config available
                        if let Err(e) = db_service
                            .update_chapter_status(download_job.chapter_id, ChapterStatus::Error)
                            .await
                        {
                            eprintln!("[DOWNLOAD-WORKER] Failed to update status to Error: {}", e);
                        }

                        return;
                    }
                };

                let result = process_single_chapter(
                    &series_ctx,
                    &chapter_info,
                    &http_client,
                    storage_client,
                    site_config,
                    &db_service,
                )
                .await;

                if let Err(e) = result {
                    eprintln!(
                        "[DOWNLOAD-WORKER] Failed to process chapter {} (ID: {}): {}",
                        download_job.chapter_number, download_job.chapter_id, e
                    );

                    if let Err(db_error) = db_service
                        .update_chapter_status(download_job.chapter_id, ChapterStatus::Error)
                        .await
                    {
                        eprintln!(
                            "[DOWNLOAD-WORKER] Double Fault: Failed to save error status: {}",
                            db_error
                        );
                    }
                }
            });
        }
    }
}
