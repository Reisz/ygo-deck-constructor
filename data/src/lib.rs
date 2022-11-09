mod card;

use bincode::Options;
pub use card::Card;

#[must_use]
pub fn bincode_options() -> impl bincode::Options {
    bincode::DefaultOptions::new()
        .with_fixint_encoding()
        .allow_trailing_bytes()
}
