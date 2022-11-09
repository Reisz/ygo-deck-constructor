use std::{
    fs::{self, File},
    io::{BufReader, BufWriter},
    os::unix::prelude::MetadataExt,
    path::Path,
};

use anyhow::{anyhow, Result};
use indicatif::{
    DecimalBytes, HumanCount, ParallelProgressIterator, ProgressIterator, ProgressStyle,
};
use rayon::prelude::*;
use serde::{ser::SerializeSeq, Serializer};
use xz2::write::XzEncoder;

use crate::reqwest_indicatif::ProgressReader;

mod reqwest_indicatif;
mod ygoprodeck;

const CARD_INFO_LOCAL: &str = "target/card_info.json";
const OUTPUT_FILE: &str = "dist/cards.bin.xz";

fn project(card: ygoprodeck::Card) -> data::Card {
    data::Card {
        id: card.id.to_string(),
        name: card.name,
        desc: card.desc,
    }
}

fn main() -> Result<()> {
    let style =
        ProgressStyle::with_template("{bar} {human_pos}/{human_len} ({eta} remaining)").unwrap();

    let cards = if Path::new(CARD_INFO_LOCAL).try_exists()? {
        println!("[1/3] Loading cards...");
        ygoprodeck::parse(BufReader::new(ProgressReader::from_path(CARD_INFO_LOCAL)?))
    } else {
        println!("[1/3] Downloading cards...");
        let response = reqwest::blocking::get(ygoprodeck::URL)?;
        ygoprodeck::parse(BufReader::new(ProgressReader::from_response(response)))
    }?;

    println!("[2/3] Converting...");
    let cards = cards
        .into_par_iter()
        .progress_with_style(style.clone())
        .map(project)
        .collect::<Vec<_>>();

    println!("[3/3] Saving...");
    let file = BufWriter::new(File::create(OUTPUT_FILE)?);

    let mut serializer = bincode::Serializer::new(XzEncoder::new(file, 9), data::bincode_options());
    let mut seq = serializer.serialize_seq(Some(cards.len()))?;
    cards
        .iter()
        .progress_with_style(style)
        .map(|v| seq.serialize_element(v).map_err(|e| anyhow!(e)))
        .collect::<Result<Vec<()>>>()?;

    println!(
        "\nSaved {} cards in {}.",
        HumanCount(cards.len().try_into()?),
        DecimalBytes(fs::metadata(OUTPUT_FILE)?.size())
    );

    Ok(())
}
