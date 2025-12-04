use askama::Template;
use async_stream::stream;
use axum::{
    extract::Form,
    response::{IntoResponse, Response, Sse},
};
use datastar::prelude::{PatchElements, PatchSignals};
use serde::Deserialize;
use serde_json::json;
use std::convert::Infallible;
use tracing::info;

#[derive(Template)]
#[template(path = "contact_form.html")]
pub struct ContactFormTemplate;

/// Contact form data structure - simplified for standard form submission
#[derive(Deserialize)]
pub struct ContactFormData {
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub service: Option<String>,
    pub message: Option<String>,
}

/// Handler to serve the contact form fragment
/// Called by @get('/contact/form')
pub async fn contact_form_handler() -> Result<Response, crate::AppError> {
    let html = ContactFormTemplate.render()?;
    Ok(create_sse_response(html).into_response())
}

/// Validate contact form and return Result with custom validation errors
pub fn validate_contact_form(form: &ContactFormData) -> Result<(), ContactFormError> {
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
pub struct ContactFormError {
    error_message: String,
}

impl ContactFormError {
    pub fn new(message: &str) -> Self {
        Self {
            error_message: message.to_string(),
        }
    }
}

/// Basic email validation using a simple check for proper format
pub fn is_valid_email(email: &str) -> bool {
    let email = email.trim();

    // Must contain @ symbol
    if !email.contains('@') {
        return false;
    }

    // Split into local and domain parts
    let parts: Vec<&str> = email.split('@').collect();
    let [local, domain] = parts.as_slice() else {
        return false;
    };

    // Validate local part
    if local.is_empty() {
        return false;
    }

    // Validate domain part
    if domain.is_empty() {
        return false;
    }

    // Domain must contain a dot for basic TLD validation
    domain.contains('.')
}

/// Handler for form submission
/// Receives form data (not JSON signals)
pub async fn contact_submit_handler(Form(form): Form<ContactFormData>) -> Response {
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
pub fn create_sse_response(html: String) -> impl IntoResponse {
    Sse::new(stream! {
        let patch = PatchElements::new(html);
        yield Ok::<_, Infallible>(patch.write_as_axum_sse_event());
    })
}

/// Basic input sanitization to prevent XSS attacks
pub fn sanitize_input(input: &str) -> String {
    let sanitized_chars = input
        .chars()
        .filter(|c| c.is_ascii() && !c.is_control());

    let sanitized: String = sanitized_chars.collect();
    sanitized.trim().to_string()
}