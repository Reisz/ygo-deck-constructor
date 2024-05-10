use std::{
    fs::{self, File},
    io::{BufRead, BufReader, BufWriter},
    os::unix::prelude::MetadataExt,
};

use anyhow::Result;
use bincode::Options;
use data_processor::{
    cache::{ensure_image_cache, update_card_info_cache, CacheResult},
    extract::Extraction,
    iter_utils::{CollectParallelWithoutErrors, IntoParProgressIterator},
    refine::{self, CardDataProxy},
    reqwest_indicatif::ProgressReader,
    step, ygoprodeck, CARD_INFO_LOCAL, OUTPUT_FILE,
};
use indicatif::{DecimalBytes, HumanCount};
use rayon::prelude::*;
use tokio::try_join;
use xz2::write::XzEncoder;

fn filter(card: &ygoprodeck::Card) -> bool {
    !matches!(
        card.card_type,
        ygoprodeck::CardType::Token | ygoprodeck::CardType::SkillCard
    )
}

#[tokio::main]
async fn main() -> Result<()> {
    let (data_result, image_result) = try_join!(update_card_info_cache(), ensure_image_cache())?;
    match data_result.merge(image_result) {
        CacheResult::StillValid => {
            println!("Nothing to do");
            return Ok(());
        }
        CacheResult::ProcessingRequired => { /* continue */ }
    }

    step("Loading cards");
    let mut reader = BufReader::new(ProgressReader::from_path(CARD_INFO_LOCAL)?);
    reader.read_line(&mut String::new())?;
    let cards = ygoprodeck::parse(reader)?;

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
