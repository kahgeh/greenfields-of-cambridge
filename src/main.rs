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
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod error;
pub use error::AppError;

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
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "greenfields_of_cambridge=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Create our router
    let app = Router::new()
        // Serve the main index.html at the root
        .route("/", get(index_handler))
        // Serve static files (CSS, JS, images, etc.)
        .nest_service("/static", ServeDir::new("static"))
        // Contact form endpoints using fragment-based approach
        .route("/contact/form", get(contact_form_handler))
        .route("/contact/form", post(contact_submit_handler));

    let addr = SocketAddr::from(([0, 0, 0, 0], 7100));
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
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
        return Err(ContactFormError {
            error_message: "Name is required".to_string(),
        });
    }
    if name.len() < 2 {
        return Err(ContactFormError {
            error_message: "Name must be at least 2 characters".to_string(),
        });
    }

    let email = form.email.trim();
    if email.is_empty() {
        return Err(ContactFormError {
            error_message: "Email is required".to_string(),
        });
    }
    if !is_valid_email(&form.email) {
        return Err(ContactFormError {
            error_message: "Please enter a valid email address".to_string(),
        });
    }

    Ok(())
}

/// Custom error type for contact form validation
#[derive(Debug)]
struct ContactFormError {
    error_message: String,
}

/// Basic email validation using a simple check for proper format
fn is_valid_email(email: &str) -> bool {
    let email = email.trim();
    if !email.contains('@') {
        return false;
    }

    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return false;
    }

    let local = parts[0];
    let domain = parts[1];

    // Basic checks for local and domain parts
    !local.is_empty() && !domain.is_empty() && domain.contains('.')
}

/// Handler for form submission
/// Receives form data (not JSON signals)
async fn contact_submit_handler(Form(form): Form<ContactFormData>) -> Response {
    log_contact_form_submission(&form);

    // Validate form
    match validate_contact_form(&form) {
        Ok(_) => {
            // Process valid form data
            log_successful_submission(&form);

            // TODO: Send email, save to database, etc.
            // For now, we'll just log it

            // Send success signals
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
        Err(validation_error) => {
            // Send error signals - preserve form fields and set error message
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
    }
}

/// Log contact form submission details
fn log_contact_form_submission(form: &ContactFormData) {
    info!("Received contact form submission:");
    info!("  Name: {}", sanitize_input(&form.name));
    info!("  Email: {}", sanitize_input(&form.email));
    info!(
        "  Phone: {:?}",
        form.phone.as_ref().map(|s| sanitize_input(s))
    );
    info!(
        "  Service: {:?}",
        form.service.as_ref().map(|s| sanitize_input(s))
    );
    info!(
        "  Message: {:?}",
        form.message.as_ref().map(|s| sanitize_input(s))
    );
}

/// Log successful form validation
fn log_successful_submission(form: &ContactFormData) {
    info!(
        "Successfully validated contact form from: {} ({})",
        sanitize_input(&form.name),
        sanitize_input(&form.email)
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
    input
        .chars()
        .filter(|c| c.is_ascii() && !c.is_control())
        .collect::<String>()
        .trim()
        .to_string()
}
