use std::{
    fs::{self, File},
    future,
    io::{BufRead, BufReader, BufWriter},
    os::unix::prelude::MetadataExt,
    time::Instant,
};

use anyhow::Result;
use bincode::Options;
use common::{card::Card, Cards};
use data_processor::{
    cache::{ensure_image_cache, update_card_info_cache, CacheResult},
    image::ImageLoader,
    ui::UiManager,
    ygoprodeck, CARD_INFO_LOCAL, OUTPUT_FILE,
};
use futures::{stream::FuturesUnordered, StreamExt, TryFutureExt};
use indicatif::{HumanBytes, HumanCount, HumanDuration};
use log::{info, warn};
use tokio::{task::spawn_blocking, try_join};
use xz2::write::XzEncoder;

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
    let loader = ImageLoader::new()?;

    info!("Processing cards");
    let stream: FuturesUnordered<_> = cards
        .into_iter()
        .filter(|card| {
            !matches!(
                card.card_type,
                ygoprodeck::CardType::Token | ygoprodeck::CardType::SkillCard
            )
        })
        .map(|card| async {
            let password = card.id;
            let (card, ()) = try_join!(
                spawn_blocking(|| Card::try_from(card)).map_err(anyhow::Error::from),
                loader.ensure_image(password)
            )?;

            Ok(card?)
        })
        .collect();
    let cards: Cards = ui
        .stream(stream)
        .filter_map(|card| {
            future::ready(card.map_err(|err: anyhow::Error| warn!("{:?}", err)).ok())
        })
        .collect()
        .await;

    info!("Saving images");
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
