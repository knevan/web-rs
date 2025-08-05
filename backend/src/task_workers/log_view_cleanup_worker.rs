use crate::database::DatabaseService;
use std::time::Duration;

pub async fn run_log_view_cleanup_worker(db_service: DatabaseService) {
    println!("[WORKER] Log view cleanup worker started");

    let mut interval = tokio::time::interval(Duration::from_secs(24 * 60 * 60));

    // The first tick from `interval` fires immediately
    // Skip it to ensure the first cleanup run after 24 hours
    interval.tick().await;

    loop {
        // Wait next tick in interval
        interval.tick().await;

        match db_service.cleanup_old_view_logs().await {
            Ok(deleted_rows) => {
                if deleted_rows > 0 {
                    println!(
                        "[WORKER] Cleaned up {} old log view entries",
                        deleted_rows
                    );
                } else {
                    println!("[WORKER] No old log view entries to clean up");
                }
            }
            Err(e) => {
                eprintln!("[WORKER] Error cleaning up log view entries: {}", e);
            }
        }
    }
}
