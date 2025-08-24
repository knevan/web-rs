use crate::app::orchestrator::repair_specific_chapter_series;
use crate::database::DatabaseService;
use crate::database::storage::StorageClient;
use crate::scraping::model::SitesConfig;
use arc_swap::ArcSwap;
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct RepairChapterMsg {
    pub series_id: i32,
    pub chapter_number: f32,
    pub new_chapter_url: String,
}

pub async fn run_repair_chapter_worker(
    mut tx: mpsc::Receiver<RepairChapterMsg>,
    db_service: DatabaseService,
    storage_client: Arc<StorageClient>,
    http_client: Client,
    sites_config: Arc<ArcSwap<SitesConfig>>,
) {
    println!("[WORKER] Repair worker started.");

    while let Some(msg) = tx.recv().await {
        println!(
            "[WORKER] Received repair request for series {} chapter {}",
            msg.series_id, msg.chapter_number
        );

        let db_clone = db_service.clone();
        let storage_clone = storage_client.clone();
        let http_clone = http_client.clone();
        let sites_config_clone = sites_config.clone();

        tokio::spawn(async move {
            let sites_config_snapshot = sites_config_clone.load();

            if let Err(e) = repair_specific_chapter_series(
                msg,
                &db_clone,
                storage_clone,
                http_clone,
                sites_config_snapshot.clone(),
            )
            .await
            {
                eprintln!("[WORKER] Failed to repair chapter: {}", e);
            }
        });
    }
}
