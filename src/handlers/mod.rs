pub mod pages;
pub mod contact;

// Re-export handlers for convenience
pub use pages::index_handler;
pub use contact::{contact_form_handler, contact_submit_handler};