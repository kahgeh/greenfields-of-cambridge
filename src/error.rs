use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    NotFound,
    BadRequest(String),
    InternalError(String),
    Render(askama::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::NotFound => write!(f, "The requested resource was not found"),
            AppError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            AppError::InternalError(msg) => write!(f, "Internal server error: {}", msg),
            AppError::Render(err) => write!(f, "Template rendering error: {}", err),
        }
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "The page you're looking for doesn't exist or has been moved."),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            AppError::InternalError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "An unexpected error occurred. Please try again later."),
            AppError::Render(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to render the page. Please try again."),
        };

        // Use the error template
        #[derive(Debug, Template)]
        #[template(path = "error.html")]
        struct ErrorTmpl {
            lang: String,
            status: u16,
            title: String,
            message: String,
        }

        let tmpl = ErrorTmpl {
            lang: "en".to_string(),
            status: status.as_u16(),
            title: match self {
                AppError::NotFound => "Page Not Found".to_string(),
                AppError::BadRequest(_) => "Bad Request".to_string(),
                AppError::InternalError(_) => "Internal Server Error".to_string(),
                AppError::Render(_) => "Rendering Error".to_string(),
            },
            message: error_message.to_string(),
        };

        match tmpl.render() {
            Ok(body) => (status, Html(body)).into_response(),
            Err(_) => {
                // Fallback if template rendering fails
                (
                    status,
                    Html(format!(
                        r#"
                        <!DOCTYPE html>
                        <html lang="en">
                        <head><title>Error {}</title></head>
                        <body>
                            <h1>Error {}</h1>
                            <p>{}</p>
                            <a href="/">Return to Home</a>
                        </body>
                        </html>
                        "#,
                        status.as_u16(),
                        status.as_u16(),
                        error_message
                    )),
                )
                    .into_response()
            }
        }
    }
}

// Implement From traits for automatic conversions
impl From<askama::Error> for AppError {
    fn from(err: askama::Error) -> Self {
        AppError::Render(err)
    }
}