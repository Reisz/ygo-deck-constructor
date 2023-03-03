use std::{
    fs::{self, File},
    io::{self, BufRead, BufReader, BufWriter, Read, Write},
    os::unix::prelude::MetadataExt,
    path::Path,
};

use anyhow::{anyhow, Result};
use clap::Parser;
use data::card::{Card, CardData, Id};
use indicatif::{
    DecimalBytes, HumanCount, ParallelProgressIterator, ProgressIterator, ProgressStyle,
};
use rayon::prelude::*;
use serde::{ser::SerializeMap, Serializer};
use xz2::write::XzEncoder;

use crate::{cli::Args, reqwest_indicatif::ProgressReader};

mod cli;
mod reqwest_indicatif;
mod ygoprodeck;

const CARD_INFO_LOCAL: &str = "target/card_info.json";
const OUTPUT_FILE: &str = "dist/cards.bin.xz";

fn filter(card: &ygoprodeck::Card) -> bool {
    !matches!(
        card.card_type,
        ygoprodeck::CardType::Token | ygoprodeck::CardType::SkillCard
    )
}

fn project(card: ygoprodeck::Card) -> (Id, Card) {
    (
        Id::new(card.id),
        Card {
            name: card.name,
            desc: card.desc,
        },
    )
}

fn step(text: &str) {
    println!("{} {text}...", console::style(">").bold().blue());
}

fn get_card_info_download() -> Result<impl Read> {
    step("Downloading cards");
    let response = reqwest::blocking::get(ygoprodeck::URL)?;
    Ok(BufReader::new(ProgressReader::from_response(response)))
}

fn get_card_info(cached: bool) -> Result<Vec<ygoprodeck::Card>> {
    if !cached {
        return ygoprodeck::parse(get_card_info_download()?);
    }

    step("Checking online database version");
    let online_version = ygoprodeck::get_version(reqwest::blocking::get(ygoprodeck::VERSION_URL)?)?;

    let mut cache_ok = Path::new(CARD_INFO_LOCAL).exists();
    if cache_ok {
        let local_version = {
            let mut tmp = String::new();
            BufReader::new(File::open(CARD_INFO_LOCAL)?).read_line(&mut tmp)?;
            tmp.truncate(tmp.len() - 1); // Remove trailing newline
            tmp
        };

        if local_version == online_version {
            println!("  Version: {local_version}");
        } else {
            println!("  Version: {local_version} (out of date, current: {online_version})");
            cache_ok = false;
        }
    }

    if !cache_ok {
        let mut local_file = BufWriter::new(File::create(CARD_INFO_LOCAL)?);
        writeln!(local_file, "{online_version}")?;
        io::copy(&mut get_card_info_download()?, &mut local_file)?;
    }

    step("Loading cards");
    let mut reader = BufReader::new(ProgressReader::from_path(CARD_INFO_LOCAL)?);
    reader.read_line(&mut String::new())?;
    ygoprodeck::parse(reader)
}

fn main() -> Result<()> {
    let args = Args::try_parse()?;

    let style =
        ProgressStyle::with_template("{bar} {human_pos}/{human_len} ({eta} remaining)").unwrap();

    let cards = get_card_info(args.cache)?;

    step("Converting");
    let cards = cards
        .into_par_iter()
        .progress_with_style(style.clone())
        .filter(filter)
        .map(project)
        .collect::<CardData>();

    step("Saving");
    let file = BufWriter::new(File::create(OUTPUT_FILE)?);

    let mut serializer = bincode::Serializer::new(XzEncoder::new(file, 9), data::bincode_options());
    let mut map = serializer.serialize_map(Some(cards.len()))?;
    cards
        .iter()
        .progress_with_style(style)
        .try_for_each(|(k, v)| map.serialize_entry(k, v).map_err(|e| anyhow!(e)))?;

    println!(
        "  Saved {} cards in {}.",
        HumanCount(cards.len().try_into()?),
        DecimalBytes(fs::metadata(OUTPUT_FILE)?.size())
    );

    Ok(())
}
