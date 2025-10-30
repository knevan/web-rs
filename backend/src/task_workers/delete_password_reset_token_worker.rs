use std::time::Duration;

use crate::database::DatabaseService;

pub async fn run_cleanup_password_reset_token_worker(db_service: DatabaseService) {
    let mut interval = tokio::time::interval(Duration::from_mins(10));

    interval.tick().await;

    loop {
        interval.tick().await;

        match db_service.cleanup_password_reset_token().await {
            Ok(delete_token) => {
                if delete_token > 0 {
                    println!("Deleting password reset token: {}", delete_token);
                }
            }
            Err(error) => {
                println!("Failed to cleanup password reset token: {}", error);
            }
        }
    }
}
