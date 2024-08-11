//! Constants for data transfer between build directories and hosted app.

use bincode::Options;

/// Name for the main data file.
pub const DATA_FILENAME: &str = "cards.bin.xz";

/// Directory for individual image files.
pub const IMAGE_DIRECTORY: &str = "images";

/// File ending for individual image files.
pub const IMAGE_FILE_ENDING: &str = "avif";

/// Bincode settings for the data file.
#[must_use]
pub fn bincode_options() -> impl bincode::Options {
    bincode::DefaultOptions::new()
        .with_fixint_encoding()
        .allow_trailing_bytes()
}
