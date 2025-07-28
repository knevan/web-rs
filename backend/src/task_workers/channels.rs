use crate::database::DatabaseService;
use crate::database::storage::StorageClient;
use crate::scraping::model::SitesConfig;
use crate::task_workers::delete_series_worker::run_deletion_background_worker;
use crate::task_workers::repair_chapter_worker::{
    RepairChapterMsg, run_repair_chapter_worker,
};
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct WorkerChannels {
    pub deletion_tx: mpsc::Sender<i32>,
    pub repair_tx: mpsc::Sender<RepairChapterMsg>,
}

pub fn setup_worker_channels(
    db_service: DatabaseService,
    storage_client: Arc<StorageClient>,
    http_client: Client,
    sites_config: Arc<SitesConfig>,
) -> WorkerChannels {
    // Deletion worker channels
    let (deletion_tx, deletion_rx) = mpsc::channel::<i32>(32);
    tokio::spawn(run_deletion_background_worker(
        deletion_rx,
        db_service.clone(),
        storage_client.clone(),
    ));

    // Repair worker channels
    let (repair_tx, repair_rx) = mpsc::channel::<RepairChapterMsg>(32);
    tokio::spawn(run_repair_chapter_worker(
        repair_rx,
        db_service.clone(),
        storage_client.clone(),
        http_client.clone(),
        sites_config.clone(),
    ));

    WorkerChannels {
        deletion_tx,
        repair_tx,
    }
}
