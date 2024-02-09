use std::{
    fs::{self, File},
    io::{self, BufRead, BufReader, BufWriter, Read, Write},
    os::unix::prelude::MetadataExt,
};

use anyhow::Result;
use bincode::Options;
use data_processor::{
    cache::{self, CacheBehavior},
    extract::Extraction,
    image::{self, save_missing_ids},
    iter_utils::{CollectParallelWithoutErrors, IntoParProgressIterator},
    print_err,
    refine::{self, CardDataProxy},
    reqwest_indicatif::ProgressReader,
    step, ygoprodeck, CARD_INFO_LOCAL, OUTPUT_FILE,
};
use indicatif::{DecimalBytes, HumanCount};
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

fn main() -> Result<()> {
    let cache = cache::get_behavior()?;
    if matches!(cache, CacheBehavior::Nothing) {
        println!("Nothing to do");
        return Ok(());
    }

    let cards = get_card_info(cache)?;

    step("Extractíng");
    let cards: Vec<Extraction> = cards
        .into_par_progress_iter()
        .filter(filter)
        .map(Extraction::try_from)
        .collect_without_errors();

    step("Refining");
    let CardDataProxy(cards) = cards
        .into_par_progress_iter()
        .map(refine::refine)
        .collect_without_errors();

    step("Checking images");
    let images = image::available_ids()?;
    let missing_images = cards
        .entries()
        .par_iter()
        .map(|(&id, _)| id)
        .filter(|id| !images.contains(id))
        .collect::<Vec<_>>();

    if !missing_images.is_empty() {
        print_err!("Missing images for {} cards", missing_images.len());
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
