use std::{
    fs::{self, File},
    io::{self, BufRead, BufReader, BufWriter, Read, Write},
    os::unix::prelude::MetadataExt,
};

use anyhow::{anyhow, Result};
use common::card::CardData;
use data_processor::{
    cache::{self, CacheBehavior},
    project::project,
    reqwest_indicatif::ProgressReader,
    step, ygoprodeck, CARD_INFO_LOCAL, OUTPUT_FILE,
};
use indicatif::{
    DecimalBytes, HumanCount, ParallelProgressIterator, ProgressIterator, ProgressStyle,
};
use rayon::prelude::*;
use serde::{ser::SerializeMap, Serializer};
use xz2::write::XzEncoder;

fn filter(card: &ygoprodeck::Card) -> bool {
    !matches!(
        card.card_type,
        ygoprodeck::CardType::Token | ygoprodeck::CardType::SkillCard
    )
}

fn get_card_info_download() -> Result<impl Read> {
    step("Downloading cards");
    let response = reqwest::blocking::get(ygoprodeck::URL)?;
    Ok(BufReader::new(ProgressReader::from_response(response)))
}

fn get_card_info(cache_behavior: CacheBehavior) -> Result<Vec<ygoprodeck::Card>> {
    if let CacheBehavior::Download { online_version } = cache_behavior {
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
    let style =
        ProgressStyle::with_template("{bar} {human_pos}/{human_len} ({eta} remaining)").unwrap();

    let cache = cache::get_behavior()?;
    if matches!(cache, CacheBehavior::Nothing) {
        println!("Nothing to do");
        return Ok(());
    }

    let cards = get_card_info(cache)?;

    step("Converting");
    let cards = cards
        .into_par_iter()
        .progress_with_style(style.clone())
        .filter(filter)
        .map(project)
        .collect::<CardData>();

    step("Saving");
    let file = BufWriter::new(File::create(OUTPUT_FILE)?);

    let mut serializer =
        bincode::Serializer::new(XzEncoder::new(file, 9), common::bincode_options());
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
