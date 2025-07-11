use crate::auth;
use crate::db::db::DatabaseService;
use axum::http::{HeaderValue, Method, header};
use axum::{Router, serve};
use lettre::AsyncSmtpTransport;
use reqwest::Client;
use std::env;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

// Type definition for Mailer
pub type Mailer = AsyncSmtpTransport<lettre::Tokio1Executor>;

#[derive(Clone)]
pub struct AppState {
    pub db_service: DatabaseService,
    pub mailer: Mailer,
    pub http_client: Client,
}

// Function to setup builder and server
pub async fn run(
    listener: TcpListener,
    db_pool: sqlx::PgPool,
    mailer: Mailer,
    http_client: Client,
) -> anyhow::Result<()> {
    // Create AppState
    let app_state = AppState {
        db_service: DatabaseService::new(db_pool),
        mailer,
        http_client,
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
    // The `auth::routes::routes()` function returns a Router with all auth-related endpoints.
    let app = Router::new()
        .merge(auth::routes::routes())
        .with_state(app_state)
        .layer(cors);

    println!("[STARTUP] Server started successfully!");

    // Run server
    serve(listener, app).await?;

    Ok(())
}
