#![feature(file_set_times)]

pub mod cache;
pub mod project;
pub mod reqwest_indicatif;
pub mod ygoprodeck;

pub const CARD_INFO_LOCAL: &str = "target/card_info.json";
pub const OUTPUT_FILE: &str = "dist/cards.bin.xz";

pub fn step(text: &str) {
    println!("{} {text}...", console::style(">").bold().blue());
}

