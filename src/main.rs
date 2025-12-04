use askama::Template;
use async_stream::stream;
use axum::{
    extract::Form,
    response::{Html, IntoResponse, Response, Sse},
    routing::{get, post},
    Router,
};
use datastar::prelude::{PatchElements, PatchSignals};
use serde::Deserialize;
use serde_json::json;
use std::{convert::Infallible, net::SocketAddr};
use tokio::signal;
use tower_http::services::ServeDir;
use tracing::{error, info};

mod error;
mod settings;
pub use error::AppError;
pub use settings::{Settings, SettingsError};

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

#[derive(Template)]
#[template(path = "contact_form.html")]
struct ContactFormTemplate;

/// Contact form data structure - simplified for standard form submission
#[derive(Deserialize)]
struct ContactFormData {
    name: String,
    email: String,
    phone: Option<String>,
    service: Option<String>,
    message: Option<String>,
}

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
        .route("/", get(index_handler))
        // Serve static files (CSS, JS, images, etc.)
        .nest_service("/static", ServeDir::new("static"))
        // Contact form endpoints using fragment-based approach
        .route("/contact/form", get(contact_form_handler))
        .route("/contact/form", post(contact_submit_handler));

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

/// Handler for the main landing page
async fn index_handler() -> Result<Html<String>, AppError> {
    let html = IndexTemplate.render()?;
    Ok(Html(html))
}

/// Handler to serve the contact form fragment
/// Called by @get('/contact/form')
async fn contact_form_handler() -> Result<Response, AppError> {
    let html = ContactFormTemplate.render()?;
    Ok(create_sse_response(html).into_response())
}

/// Validate contact form and return Result with custom validation errors
fn validate_contact_form(form: &ContactFormData) -> Result<(), ContactFormError> {
    let name = form.name.trim();

    if name.is_empty() {
        return Err(ContactFormError::new("Name is required"));
    }

    if name.len() < 2 {
        return Err(ContactFormError::new("Name must be at least 2 characters"));
    }

    let email = form.email.trim();

    if email.is_empty() {
        return Err(ContactFormError::new("Email is required"));
    }

    if !is_valid_email(&form.email) {
        return Err(ContactFormError::new("Please enter a valid email address"));
    }

    Ok(())
}

/// Custom error type for contact form validation
#[derive(Debug)]
struct ContactFormError {
    error_message: String,
}

impl ContactFormError {
    fn new(message: &str) -> Self {
        Self {
            error_message: message.to_string(),
        }
    }
}

/// Basic email validation using a simple check for proper format
fn is_valid_email(email: &str) -> bool {
    let email = email.trim();

    if !email.contains('@') {
        return false;
    }

    let parts: Vec<&str> = email.split('@').collect();
    let [local, domain] = parts.as_slice() else {
        return false;
    };

    if local.is_empty() {
        return false;
    }

    if domain.is_empty() {
        return false;
    }

    domain.contains('.')
}

/// Handler for form submission
/// Receives form data (not JSON signals)
async fn contact_submit_handler(Form(form): Form<ContactFormData>) -> Response {
    log_contact_form_submission(&form);

    // Validate form and return early if invalid
    if let Err(validation_error) = validate_contact_form(&form) {
        return create_error_response(&form, validation_error);
    }

    // Process valid form data
    log_successful_submission(&form);

    // TODO: Send email, save to database, etc.
    // For now, we'll just log it

    create_success_response()
}

fn create_success_response() -> Response {
    let signals = json!({
        "showSuccess": true,
        "showError": false,
        "errorMessage": "",
        // Reset form fields
        "name": "",
        "email": "",
        "phone": "",
        "service": "",
        "message": ""
    });

    Sse::new(stream! {
        let patch = PatchSignals::new(signals.to_string());
        yield Ok::<_, Infallible>(patch.write_as_axum_sse_event());
    })
    .into_response()
}

fn create_error_response(form: &ContactFormData, validation_error: ContactFormError) -> Response {
    let escaped_error = validation_error.error_message.replace('"', "\\\"");
    let signals = json!({
        "showSuccess": false,
        "showError": true,
        "errorMessage": escaped_error,
        // Preserve form fields
        "name": sanitize_input(&form.name),
        "email": sanitize_input(&form.email),
        "phone": form.phone.as_ref().map(|s| sanitize_input(s)).unwrap_or_default(),
        "service": form.service.as_ref().map(|s| sanitize_input(s)).unwrap_or_default(),
        "message": form.message.as_ref().map(|s| sanitize_input(s)).unwrap_or_default()
    });

    Sse::new(stream! {
        let patch = PatchSignals::new(signals.to_string());
        yield Ok::<_, Infallible>(patch.write_as_axum_sse_event());
    })
    .into_response()
}

/// Log contact form submission details
fn log_contact_form_submission(form: &ContactFormData) {
    let sanitized_fields = FormLogFields {
        name: sanitize_input(&form.name),
        email: sanitize_input(&form.email),
        phone: form.phone.as_ref().map(|s| sanitize_input(s)),
        service: form.service.as_ref().map(|s| sanitize_input(s)),
        message: form.message.as_ref().map(|s| sanitize_input(s)),
    };

    info!(
        "Received contact form submission: Name: {}, Email: {}, Phone: {:?}, Service: {:?}, Message: {:?}",
        sanitized_fields.name,
        sanitized_fields.email,
        sanitized_fields.phone,
        sanitized_fields.service,
        sanitized_fields.message
    );
}

struct FormLogFields {
    name: String,
    email: String,
    phone: Option<String>,
    service: Option<String>,
    message: Option<String>,
}

/// Log successful form validation
fn log_successful_submission(form: &ContactFormData) {
    let sanitized_name = sanitize_input(&form.name);
    let sanitized_email = sanitize_input(&form.email);

    info!(
        "Successfully validated contact form from: {} ({})",
        sanitized_name,
        sanitized_email
    );
}

/// Create SSE response with content
fn create_sse_response(html: String) -> impl IntoResponse {
    Sse::new(stream! {
        let patch = PatchElements::new(html);
        yield Ok::<_, Infallible>(patch.write_as_axum_sse_event());
    })
}

/// Basic input sanitization to prevent XSS attacks
fn sanitize_input(input: &str) -> String {
    let sanitized: String = input
        .chars()
        .filter(|c| c.is_ascii() && !c.is_control())
        .collect();

    sanitized.trim().to_string()
}
