use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, BufRead, BufReader, BufWriter, Read, Write},
    os::unix::prelude::MetadataExt,
};

use anyhow::Result;
use bincode::Options;
use common::{
    card::{Card, Id},
    card_data::CardData,
};
use data_processor::{
    cache::{self, CacheBehavior},
    default_progress_style, extract,
    image::{self, save_missing_ids},
    refine,
    reqwest_indicatif::ProgressReader,
    step, ygoprodeck, CARD_INFO_LOCAL, OUTPUT_FILE,
};
use indicatif::{DecimalBytes, HumanCount, ParallelProgressIterator};
use rayon::prelude::*;
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

fn to_card_data(iter: impl ParallelIterator<Item = (Vec<Id>, Card)>) -> CardData {
    type Entries = HashMap<Id, Card>;
    type Ids = Vec<(Id, Vec<Id>)>;

    let (entries, ids): (Entries, Ids) = iter
        .map(|(mut ids, card)| {
            let id = ids.remove(0);
            ((id, card), (id, ids))
        })
        .unzip();

    let alternatives = ids
        .into_iter()
        .flat_map(|(id, ids)| ids.into_iter().map(move |src| (src, id)))
        .collect();

    CardData::new(entries, alternatives)
}

fn main() -> Result<()> {
    let style = default_progress_style();

    let cache = cache::get_behavior()?;
    if matches!(cache, CacheBehavior::Nothing) {
        println!("Nothing to do");
        return Ok(());
    }

    let cards = get_card_info(cache)?;

    step("Extractíng");
    let cards = cards
        .into_par_iter()
        .progress_with_style(style.clone())
        .filter(filter)
        .filter_map(extract::extract)
        .collect::<Vec<_>>();

    step("Refining");
    let cards = to_card_data(
        cards
            .into_par_iter()
            .progress_with_style(style.clone())
            .filter_map(refine::refine),
    );

    step("Checking images");
    let images = image::available_ids()?;
    let missing_images = cards
        .entries()
        .par_iter()
        .filter(|(id, _)| !images.contains(id))
        .map(|(&id, _)| id)
        .collect::<Vec<_>>();

    if !missing_images.is_empty() {
        eprintln!("! Missing images for {} cards", missing_images.len());
        save_missing_ids(&missing_images)?;
    }

    step("Saving");
    let file = BufWriter::new(File::create(OUTPUT_FILE)?);
    common::bincode_options().serialize_into(XzEncoder::new(file, 9), &cards)?;

    println!(
        "  Saved {} cards in {}.",
        HumanCount(cards.entries().len().try_into()?),
        DecimalBytes(fs::metadata(OUTPUT_FILE)?.size())
    );

    Ok(())
}
