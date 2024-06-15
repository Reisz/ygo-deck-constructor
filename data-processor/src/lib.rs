pub mod cache;
pub mod error;
pub mod extract;
pub mod image;
pub mod iter_utils;
pub mod refine;
pub mod reqwest_indicatif;
pub mod ygoprodeck;

/// Location of the data used by the app.
pub const OUTPUT_FILE: &str = "dist/cards.bin.xz";

/// Location of the cached card artworks.
///
/// This is part of the distribution, so clean builds can download it from the hosted app instead
/// of downloading and processing all the images individually every time.
///
/// This needs to be synced with the url below.
pub const IMAGE_CACHE: &str = "dist/images.zip";

/// Location of final images.
pub const IMAGE_DIRECTORY: &str = "dist/images";

/// Url of the image cache.
///
/// This needs to be synced with the synced with the file name above.
pub const IMAGE_CACHE_URL: &str = "https://reisz.github.io/ygo-deck-constructor/images.zip";

// Location of the cached card data download.
pub const CARD_INFO_LOCAL: &str = "target/card_info.json";

pub fn step(text: &str) {
    println!("{} {text}...", console::style(">").bold().blue());
}

pub fn print_err(err: &anyhow::Error) {
    let message = format!("{err:?}");
    let mut lines = message.lines();

    if let Some(line) = lines.next() {
        println!("{} {line}", console::style("!").bold().red());
    }

    for line in lines {
        println!("  {line}");
    }
}
