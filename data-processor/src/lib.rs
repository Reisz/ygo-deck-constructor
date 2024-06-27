pub mod cache;
pub mod error;
pub mod extract;
pub mod image;
pub mod ui;
pub mod ygoprodeck;

/// Location of the data used by the app.
pub const OUTPUT_FILE: &str = "dist/cards.bin.xz";

// Location of the cached card data download.
pub const CARD_INFO_LOCAL: &str = "target/card_info.json";
