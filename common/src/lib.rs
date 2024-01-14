pub mod card;
pub mod card_data;

use bincode::Options;

#[must_use]
pub fn bincode_options() -> impl bincode::Options {
    bincode::DefaultOptions::new()
        .with_fixint_encoding()
        .allow_trailing_bytes()
}
