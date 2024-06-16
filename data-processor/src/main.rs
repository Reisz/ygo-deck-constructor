use std::{
    fs::{self, File},
    io::{BufRead, BufReader, BufWriter},
    os::unix::prelude::MetadataExt,
    sync::Arc,
};

use anyhow::Result;
use bincode::Options;
use data_processor::{
    cache::{ensure_image_cache, update_card_info_cache, CacheResult},
    extract::Extraction,
    image::ImageLoader,
    iter_utils::CollectWithoutErrors,
    print_err,
    refine::{self, CardDataProxy},
    reqwest_indicatif::ProgressReader,
    step, ygoprodeck, CARD_INFO_LOCAL, OUTPUT_FILE,
};
use indicatif::{DecimalBytes, HumanCount, ProgressBar, ProgressStyle};
use tokio::{task::JoinSet, try_join};
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
    if let CacheResult::StillValid = data_result.merge(image_result) {
        println!("Nothing to do");
        return Ok(());
    }

    step("Loading cards");
    let mut reader = BufReader::new(ProgressReader::from_path(CARD_INFO_LOCAL)?);
    reader.read_line(&mut String::new())?;
    let cards = ygoprodeck::parse(reader)?;

    step("Checking images");
    let loader = Arc::new(ImageLoader::new()?);

    step("Processing cards");
    let mut downloads = JoinSet::new();
    let progress = ProgressBar::new(cards.len().try_into()?).with_style(
        ProgressStyle::with_template("{bar} {human_pos}/{human_len} ({eta} remaining)").unwrap(),
    );
    let CardDataProxy(cards) = cards
        .into_iter()
        .filter(filter)
        .map(|card| {
            let extraction = Extraction::try_from(card)?;

            let id = *extraction.ids.first().unwrap();
            let loader = Arc::clone(&loader);
            downloads.spawn(async move { loader.ensure_image(id).await });

            refine::refine(extraction)
        })
        .collect_without_errors();

    let mut errors = Vec::new();
    while let Some(result) = downloads.join_next().await {
        progress.inc(1);
        if let Err(err) = result? {
            errors.push(err);
        }
    }
    for err in errors {
        print_err(&err);
    }

    step("Processing images");
    loader.finish().await?;

    step("Saving cards");
    let file = BufWriter::new(File::create(OUTPUT_FILE)?);
    common::bincode_options().serialize_into(XzEncoder::new(file, 9), &cards)?;

    println!(
        "  Saved {} cards in {}.",
        HumanCount(cards.entries().len().try_into()?),
        DecimalBytes(fs::metadata(OUTPUT_FILE)?.size())
    );

    Ok(())
}
