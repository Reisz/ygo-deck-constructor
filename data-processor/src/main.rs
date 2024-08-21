use std::{
    fs::{self, File},
    future,
    io::{BufReader, BufWriter},
    os::unix::prelude::MetadataExt,
    path::PathBuf,
    time::Instant,
};

use anyhow::Result;
use bincode::Options;
use common::{card::FullCard, card_data::CardDataStorage, transfer};
use data_processor::{
    cache::{
        ensure_image_cache, update_card_info_cache, CacheResult, CARD_INFO_LOCAL, CARD_STAPLES,
    },
    image::ImageLoader,
    ui::UiManager,
    ygoprodeck, OUTPUT_DIRECTORY,
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
    let cards = ygoprodeck::parse(BufReader::new(File::open(CARD_INFO_LOCAL)?))?;
    let staples = ygoprodeck::parse(BufReader::new(File::open(CARD_STAPLES)?))?;

    info!("Checking images");
    let loader = ImageLoader::new()?;

    info!("Processing cards");
    let stream: FuturesUnordered<_> = cards
        .into_iter()
        .filter(|card| card.card_type != "Token" && card.card_type != "Skill Card")
        .map(|card| async {
            let password = card.id;
            let (card, ()) = try_join!(
                spawn_blocking(|| FullCard::try_from(card)).map_err(anyhow::Error::from),
                loader.ensure_image(password)
            )?;

            Ok(card?)
        })
        .collect();
    let cards = ui
        .stream(stream)
        .filter_map(|card| {
            future::ready(card.map_err(|err: anyhow::Error| warn!("{:?}", err)).ok())
        })
        .collect::<Vec<_>>()
        .await;
    let count = cards.len();

    let staples = staples.into_iter().map(|card| card.id).collect();
    let data = CardDataStorage::new(cards, staples);

    info!("Saving images");
    loader.finish().await?;

    info!("Saving cards");
    let path = &PathBuf::from(OUTPUT_DIRECTORY).join(transfer::DATA_FILENAME);
    let prev_size = fs::metadata(path).ok().map(|meta| meta.size());

    let saving_start = Instant::now();
    let file = BufWriter::new(File::create(path)?);
    transfer::bincode_options().serialize_into(XzEncoder::new(file, 9), &data)?;
    let elapsed = saving_start.elapsed();
    let size = fs::metadata(path)?.size();

    info!(
        "Saved {} cards ({} in {}).",
        HumanCount(count.try_into().unwrap()),
        HumanBytes(fs::metadata(path)?.size()),
        HumanDuration(elapsed)
    );

    if let Some(prev_size) = prev_size {
        let change = (size as f64 - prev_size as f64) * 100.0 / prev_size as f64;
        if change.abs() > 0.5 {
            info!(
                "Previous size: {} (Change: {change:+.2}%)",
                HumanBytes(prev_size)
            );
        }
    }

    Ok(())
}
