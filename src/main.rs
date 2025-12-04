use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tokio::signal;
use tower_http::services::ServeDir;
use tracing::{error, info};

mod error;
mod settings;
mod handlers;

pub use error::AppError;
pub use settings::{Settings, SettingsError};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize settings
    Settings::initialize()?;
    let settings = Settings::get();

    // Configure tracing based on settings
    let env_filter = match tracing_subscriber::EnvFilter::try_from_default_env() {
        Ok(filter) => filter,
        Err(_) => {
            // Use log level from settings
            let default_filter = format!("debug,tower_http={}", settings.log.level);
            default_filter.parse().expect("Invalid filter directive")
        }
    };

    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .init();

    info!(
        "Starting {} v{} on {}:{}",
        settings.metadata.name,
        settings.metadata.version,
        settings.server.host,
        settings.server.port
    );

    // Create our router
    let app = Router::new()
        // Serve the main index.html at the root
        .route("/", get(handlers::index_handler))
        // Serve static files (CSS, JS, images, etc.)
        .nest_service("/static", ServeDir::new("static"))
        // Contact form endpoints using fragment-based approach
        .route("/contact/form", get(handlers::contact_form_handler))
        .route("/contact/form", post(handlers::contact_submit_handler));

    let addr = format!("{}:{}", settings.server.host, settings.server.port)
        .parse::<SocketAddr>()?;

    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Listening on {}", addr);

    // Start the server with graceful shutdown
    if let Err(e) = axum::serve(listener, app)
        .with_graceful_shutdown(wait_for_shutdown_signal())
        .await
    {
        error!("Server error: {}", e);
        return Err(e.into());
    }

    info!("Server has shut down gracefully");
    Ok(())
}

/// Wait for shutdown signals (SIGINT or SIGTERM)
async fn wait_for_shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
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
        () = ctrl_c => {
            info!("Received Ctrl+C (SIGINT) signal");
        },
        () = terminate => {
            info!("Received SIGTERM signal");
        },
    }
}