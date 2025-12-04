use askama::Template;
use axum::response::Html;
use crate::AppError;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate;

/// Handler for the main landing page
pub async fn index_handler() -> Result<Html<String>, AppError> {
    let html = IndexTemplate.render()?;
    Ok(Html(html))
}