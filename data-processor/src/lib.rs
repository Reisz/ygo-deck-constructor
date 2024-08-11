pub mod cache;
pub mod error;
pub mod extract;
pub mod image;
pub mod ui;
pub mod ygoprodeck;

/// Output location for app data.
pub const OUTPUT_DIRECTORY: &str = "dist";

/// Deployment URL.
pub const URL: &str = "https://reisz.github.io/ygo-deck-constructor";
