use std::sync::Arc;

use arc_swap::ArcSwap;
use reqwest::Client;
use tokio::sync::mpsc;

use crate::database::DatabaseService;
use crate::database::storage::StorageClient;
use crate::scraping::model::SitesConfig;
use crate::task_workers::delete_series_worker::{
    run_deletion_scheduler, run_deletion_worker,
};
use crate::task_workers::log_view_cleanup_worker::run_log_view_cleanup_worker;
use crate::task_workers::repair_chapter_worker::{
    RepairChapterMsg, run_repair_chapter_worker,
};
use crate::task_workers::series_check_worker::{
    SeriesCheckJob, run_series_check_scheduler, run_series_check_worker,
};

#[derive(Clone)]
pub struct OnDemandChannels {
    pub repair_tx: mpsc::Sender<RepairChapterMsg>,
    pub series_check_tx: async_channel::Sender<SeriesCheckJob>,
}

pub fn setup_worker_channels(
    db_service: DatabaseService,
    storage_client: Arc<StorageClient>,
    http_client: Client,
    sites_config: Arc<ArcSwap<SitesConfig>>,
) -> OnDemandChannels {
    // Check series worker channels
    let (series_check_tx, series_check_rx) = async_channel::bounded::<SeriesCheckJob>(16);

    tokio::spawn(run_series_check_scheduler(
        db_service.clone(),
        series_check_tx.clone(),
    ));

    const SERIES_CHECK_WORKER_COUNT: usize = 3;
    for i in 0..SERIES_CHECK_WORKER_COUNT {
        let rx_clone = series_check_rx.clone();
        tokio::spawn(run_series_check_worker(
            i,
            db_service.clone(),
            storage_client.clone(),
            http_client.clone(),
            sites_config.clone(),
            rx_clone,
        ));
    }

    // Deletion worker channels
    let (deletion_tx, deletion_rx) = mpsc::channel(16);

    tokio::spawn(run_deletion_scheduler(db_service.clone(), deletion_tx));
    tokio::spawn(run_deletion_worker(
        1,
        db_service.clone(),
        storage_client.clone(),
        deletion_rx,
    ));

    // Repair worker channels
    let (repair_tx, repair_rx) = mpsc::channel::<RepairChapterMsg>(16);
    tokio::spawn(run_repair_chapter_worker(
        repair_rx,
        db_service.clone(),
        storage_client.clone(),
        http_client.clone(),
        sites_config.clone(),
    ));

    // Log View Cleanup worker
    tokio::spawn(run_log_view_cleanup_worker(db_service.clone()));

    OnDemandChannels {
        repair_tx,
        series_check_tx,
    }
}
