pub mod card;
pub mod card_data;

use bincode::Options;

/// Directory for individual images in deployment.
pub const IMAGE_DIRECTORY: &str = "images";

/// File ending used by image files.
pub const IMAGE_FILE_ENDING: &str = "avif";

pub type Cards = Vec<card::Card>;

#[must_use]
pub fn bincode_options() -> impl bincode::Options {
    bincode::DefaultOptions::new()
        .with_fixint_encoding()
        .allow_trailing_bytes()
}
