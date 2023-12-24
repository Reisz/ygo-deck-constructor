use indicatif::ProgressStyle;

pub mod cache;
pub mod error;
pub mod extract;
pub mod image;
pub mod refine;
pub mod reqwest_indicatif;
pub mod ygoprodeck;

pub const CARD_INFO_LOCAL: &str = "data/card_info.json";
pub const OUTPUT_FILE: &str = "dist/cards.bin.xz";
pub const MISSING_IMAGES: &str = "data/missing_images.bin";

#[must_use]
pub fn default_progress_style() -> ProgressStyle {
    ProgressStyle::with_template("{bar} {human_pos}/{human_len} ({eta} remaining)").unwrap()
}

pub fn step(text: &str) {
    println!("{} {text}...", console::style(">").bold().blue());
}
