use crate::auth;
use crate::database::DatabaseService;
use crate::database::storage::StorageClient;
use crate::scraping::model::SitesConfig;
use crate::task_workers::channels::{OnDemandChannels, setup_worker_channels};
use axum::http::{HeaderValue, Method, header};
use axum::{Router, serve};
use lettre::AsyncSmtpTransport;
use reqwest::Client;
use std::env;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::cors::CorsLayer;

// Type definition for Mailer
pub type Mailer = AsyncSmtpTransport<lettre::Tokio1Executor>;

#[derive(Clone)]
pub struct AppState {
    pub db_service: DatabaseService,
    pub mailer: Mailer,
    pub http_client: Client,
    pub sites_config: Arc<SitesConfig>,
    pub storage_client: Arc<StorageClient>,
    pub worker_channels: OnDemandChannels,
}

// Function to set up builder and server
pub async fn run(
    listener: TcpListener,
    db_pool: sqlx::PgPool,
    mailer: Mailer,
    http_client: Client,
) -> anyhow::Result<()> {
    let storage_client_env = StorageClient::new_from_env().await?;
    let storage_client = Arc::new(storage_client_env);

    let db_service = DatabaseService::new(db_pool);
    let sites_config =
        Arc::new(SitesConfig::load("backend/config_sites.toml")?);

    // Create channels
    let worker_channels = setup_worker_channels(
        db_service.clone(),
        storage_client.clone(),
        http_client.clone(),
        sites_config.clone(),
    );

    // Create AppState
    let app_state = AppState {
        db_service,
        mailer,
        http_client,
        sites_config,
        storage_client,
        worker_channels,
    };

    // CORS Configuration
    let frontend_origin = env::var("FRONTEND_ORIGIN")
        .unwrap_or_else(|_| "http://localhost:5173".to_string());

    let cors = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
        ])
        .allow_origin(
            frontend_origin
                .parse::<HeaderValue>()
                .expect("Invalid frontend origin"),
        )
        .allow_credentials(true)
        .allow_headers([
            header::AUTHORIZATION,
            header::ACCEPT,
            header::CONTENT_TYPE,
            header::COOKIE,
            header::ORIGIN,
        ]);

    // Setup App router
    // Initialize the router and attach the authentication routes
    let app = Router::new()
        .merge(auth::routes::routes())
        .with_state(app_state)
        .layer(cors);

    println!("[STARTUP] Server started successfully!");

    // Run server
    serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("[STARTUP] Signal received, starting graceful shutdown");
}
