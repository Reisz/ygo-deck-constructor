use std::{
    fs::{self, File},
    io::{BufRead, BufReader, BufWriter},
    os::unix::prelude::MetadataExt,
    sync::Arc,
    time::Instant,
};

use anyhow::Result;
use bincode::Options;
use common::Cards;
use data_processor::{
    cache::{ensure_image_cache, update_card_info_cache, CacheResult},
    extract::Extraction,
    image::ImageLoader,
    refine::{self},
    ui::UiManager,
    ygoprodeck, CARD_INFO_LOCAL, OUTPUT_FILE,
};
use indicatif::{HumanBytes, HumanCount, HumanDuration};
use log::{info, warn};
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
    let ui = UiManager::new();

    let (data_result, image_result) =
        try_join!(update_card_info_cache(&ui), ensure_image_cache(&ui))?;
    if let CacheResult::StillValid = data_result.merge(image_result) {
        println!("Nothing to do");
        return Ok(());
    }

    info!("Loading cards");
    let mut reader = BufReader::new(File::open(CARD_INFO_LOCAL)?);
    reader.read_line(&mut String::new())?;
    let cards = ygoprodeck::parse(reader)?;

    info!("Checking images");
    let loader = Arc::new(ImageLoader::new()?);

    info!("Processing cards");
    let mut downloads = JoinSet::new();
    let progress = ui.make_progress(cards.len().try_into()?);
    let cards: Cards = cards
        .into_iter()
        .filter(filter)
        .map(|card| {
            let extraction = Extraction::try_from(card)?;

            let id = *extraction.ids.first().unwrap();
            let loader = Arc::clone(&loader);
            downloads.spawn(async move { loader.ensure_image(id).await });

            refine::refine(extraction)
        })
        .filter_map(|result| {
            result
                .map_err(|err| warn!("{:?}", anyhow::Error::from(err)))
                .ok()
        })
        .collect();

    while let Some(result) = downloads.join_next().await {
        progress.inc(1);
        if let Err(err) = result? {
            warn!("{err:?}");
        }
    }
    progress.finish_and_clear();

    info!("Processing images");
    loader.finish().await?;

    info!("Saving cards");
    let saving_start = Instant::now();
    let file = BufWriter::new(File::create(OUTPUT_FILE)?);
    common::bincode_options().serialize_into(XzEncoder::new(file, 9), &cards)?;

    info!(
        "Saved {} cards ({} in {}).",
        HumanCount(cards.len().try_into()?),
        HumanBytes(fs::metadata(OUTPUT_FILE)?.size()),
        HumanDuration(saving_start.elapsed())
    );

    Ok(())
}
