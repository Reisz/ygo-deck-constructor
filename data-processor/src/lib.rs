pub mod cache;
pub mod error;
pub mod extract;
pub mod image;
pub mod ui;
pub mod ygoprodeck;

/// Location of the data used by the app.
pub const OUTPUT_FILE: &str = "dist/cards.bin.xz";

/// Location of final images.
pub const IMAGE_DIRECTORY: &str = "dist/images";

/// Location of the cached card artworks.
///
/// This is part of the distribution, so clean builds can download it from the hosted app instead
/// of downloading and processing all the images individually every time.
///
/// This needs to be synced with the URL below.
pub const IMAGE_CACHE: &str = "dist/images.zip";

/// URL of the image cache.
///
/// This needs to be synced with the synced with the file name above.
pub const IMAGE_CACHE_URL: &str = "https://reisz.github.io/ygo-deck-constructor/images.zip";

// Location of the cached card data download.
pub const CARD_INFO_LOCAL: &str = "target/card_info.json";
