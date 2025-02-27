use std::{
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use anyhow::Result;
use common::transfer;
use log::info;
use serde::Deserialize;
use tokio::{
    fs::{self, File},
    io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
    try_join,
};

use crate::{OUTPUT_DIRECTORY, URL, image, ui::UiManager, ygoprodeck};

/// Location of the cached card data download.
pub const CARD_INFO_VERSION: &str = "target/card_info_version.txt";
pub const CARD_INFO_LOCAL: &str = "target/card_info.json";
pub const CARD_STAPLES: &str = "target/card_staples.json";

#[derive(Debug, Clone, Copy)]
pub enum CacheResult {
    StillValid,
    ProcessingRequired,
}

impl CacheResult {
    #[must_use]
    pub fn merge(self, other: CacheResult) -> CacheResult {
        match (self, other) {
            (Self::StillValid, Self::StillValid) => Self::StillValid,
            _ => Self::ProcessingRequired,
        }
    }
}

async fn get_modification_time(path: impl AsRef<Path>) -> Result<SystemTime> {
    Ok(fs::metadata(path).await?.modified()?)
}

/// Ensure that the cached card info is up to date.
///
/// This function also determines based on the state of the card info cache file and output file whether processing is required.
pub async fn update_card_info_cache(ui: &UiManager) -> Result<CacheResult> {
    // Download a new version if required. Request processing in that case.
    if let Some(version) = should_download_card_info().await? {
        let write_version = async {
            let mut file = BufWriter::new(File::create(CARD_INFO_VERSION).await?);
            file.write_all(version.as_bytes()).await?;
            file.flush().await?;
            Ok::<_, anyhow::Error>(())
        };

        let database_download = async {
            let mut download = BufReader::new(ui.get("Card Database", ygoprodeck::URL).await?);
            let mut file = BufWriter::new(File::create(CARD_INFO_LOCAL).await?);
            tokio::io::copy(&mut download, &mut file).await?;
            file.flush().await?;
            Ok(())
        };

        let staple_download = async {
            let url = format!("{}?staple=yes", ygoprodeck::URL);
            let mut download = BufReader::new(ui.get("Staple Card List", url).await?);
            let mut file = BufWriter::new(File::create(CARD_STAPLES).await?);
            tokio::io::copy(&mut download, &mut file).await?;
            file.flush().await?;
            Ok(())
        };

        try_join!(write_version, database_download, staple_download)?;
        return Ok(CacheResult::ProcessingRequired);
    }

    let output_path = &PathBuf::from(OUTPUT_DIRECTORY).join(transfer::DATA_FILENAME);

    // If the output file is missing, request processing.
    if !output_path.try_exists()? {
        return Ok(CacheResult::ProcessingRequired);
    }

    // If this executable is newer than the output file, request processing.
    // This enables code changes to this executable to apply immediately.
    let (output_date, executable_date) = try_join!(
        get_modification_time(output_path),
        get_modification_time(std::env::current_exe()?)
    )?;
    if executable_date > output_date {
        return Ok(CacheResult::ProcessingRequired);
    }

    // Otherwise the output file should be up-to-date.
    Ok(CacheResult::StillValid)
}

/// Make sure the image cache exists.
pub async fn ensure_image_cache(ui: &UiManager) -> Result<CacheResult> {
    let path = &PathBuf::from(OUTPUT_DIRECTORY).join(image::CACHE_FILENAME);
    let url = &format!("{URL}/{}", image::CACHE_FILENAME);

    if path.try_exists()? {
        return Ok(CacheResult::StillValid);
    }

    let response = ui.get("Image Cache", url).await?;
    let mut response = BufReader::new(response);
    let mut file = BufWriter::new(File::create(path).await?);
    tokio::io::copy(&mut response, &mut file).await?;

    Ok(CacheResult::ProcessingRequired)
}

/// Determine whether a new version of the card info cache needs to be downloaded.
///
/// This happens either because the cache is missing or has a different version. If neither is the case, this function will return `None`, otherwise it returns the version of the online data, so it can be stored with the new data.
async fn should_download_card_info() -> Result<Option<String>> {
    const VERSION_CHECK_INTERVAL: Duration = Duration::from_secs(60 * 60 * 24); // 1 day

    // If there is no cache, just get the online version and return.
    if !Path::new(CARD_INFO_LOCAL).try_exists()? || !Path::new(CARD_INFO_VERSION).try_exists()? {
        return Ok(Some(get_online_version().await?));
    }

    // The cache file exists. Check duration since the last update (via modification date).
    if get_modification_time(CARD_INFO_VERSION).await?.elapsed()? > VERSION_CHECK_INTERVAL {
        let (online_version, local_version) = try_join!(get_online_version(), get_local_version())?;
        if local_version != online_version {
            return Ok(Some(online_version));
        }

        // The check succeeded. Update modification time so we do not keep bugging the database.
        std::fs::File::open(CARD_INFO_VERSION)?.set_modified(SystemTime::now())?;
    }

    // The cache does not need to be updated.
    Ok(None)
}

async fn get_online_version() -> Result<String> {
    pub const VERSION_URL: &str = "https://db.ygoprodeck.com/api/v7/checkDBVer.php";

    #[derive(Debug, Deserialize)]
    pub struct VersionInfo {
        pub database_version: String,
    }

    info!("Checking online database version");
    let [info]: [VersionInfo; 1] = reqwest::get(VERSION_URL)
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(info.database_version)
}

async fn get_local_version() -> Result<String> {
    let mut tmp = String::new();
    BufReader::new(File::open(CARD_INFO_VERSION).await?)
        .read_to_string(&mut tmp)
        .await?;
    Ok(tmp)
}
