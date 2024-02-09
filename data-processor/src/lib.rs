pub mod cache;
pub mod error;
pub mod extract;
pub mod image;
pub mod iter_utils;
pub mod refine;
pub mod reqwest_indicatif;
pub mod ygoprodeck;

pub const CARD_INFO_LOCAL: &str = "target/card_info.json";
pub const OUTPUT_FILE: &str = "dist/cards.bin.xz";
pub const MISSING_IMAGES: &str = "target/missing_images.bin";

pub fn step(text: &str) {
    println!("{} {text}...", console::style(">").bold().blue());
}

#[macro_export]
macro_rules! print_err {
    ($($t: tt)*) => {
        let message = format!($($t)*);
        let mut lines = message.lines();

        if let Some(line) = lines.next() {
            println!("{} {line}", console::style("!").bold().red());
        }

        for line in lines {
            println!("  {line}");
        }
    }
}
