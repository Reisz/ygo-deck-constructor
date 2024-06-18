use std::{
    path::Path,
    time::{Duration, SystemTime},
};

use anyhow::Result;
use log::info;
use serde::Deserialize;
use tokio::{
    fs::{self, File},
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    try_join,
};

use crate::{
    ui::UiManager, ygoprodeck, CARD_INFO_LOCAL, IMAGE_CACHE, IMAGE_CACHE_URL, OUTPUT_FILE,
};

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
        let database_download = async {
            Ok(BufReader::new(
                ui.get("Card Database", ygoprodeck::URL).await?,
            ))
        };

        let database_cache_writer = async {
            let mut file = BufWriter::new(File::create(CARD_INFO_LOCAL).await?);
            file.write_all(version.as_bytes()).await?;
            file.write_all("\n".as_bytes()).await?;
            Ok::<_, anyhow::Error>(file)
        };

        let (mut response, mut file) = try_join!(database_download, database_cache_writer)?;
        tokio::io::copy(&mut response, &mut file).await?;

        return Ok(CacheResult::ProcessingRequired);
    }

    // If the output file is missing, request processing.
    if !Path::new(OUTPUT_FILE).try_exists()? {
        return Ok(CacheResult::ProcessingRequired);
    }

    // If this executable is newer than the output file, request processing.
    // This enables code changes to this executable to apply immediately.
    let (output_date, executable_date) = try_join!(
        get_modification_time(OUTPUT_FILE),
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
    if Path::new(IMAGE_CACHE).try_exists()? {
        return Ok(CacheResult::StillValid);
    }

    let response = ui.get("Image Cache", IMAGE_CACHE_URL).await?;
    let mut response = BufReader::new(response);
    let mut file = BufWriter::new(File::create(IMAGE_CACHE).await?);
    tokio::io::copy(&mut response, &mut file).await?;

    Ok(CacheResult::ProcessingRequired)
}

/// Determine whether a new version of the card info cache needs to be downloaded.
///
/// This happens either because the cache is missing or has a different version. If neither is the case, this function will return `None`, otherwise it returns the version of the online data, so it can be stored with the new data.
async fn should_download_card_info() -> Result<Option<String>> {
    const VERSION_CHECK_INTERVAL: Duration = Duration::from_secs(60 * 60 * 24); // 1 day

    // If there is no cache, just get the online version and return.
    if !Path::new(CARD_INFO_LOCAL).try_exists()? {
        return Ok(Some(get_online_version().await?));
    }

    // The cache file exists. Check duration since the last update (via modification date).
    if get_modification_time(CARD_INFO_LOCAL).await?.elapsed()? > VERSION_CHECK_INTERVAL {
        let (online_version, local_version) = try_join!(get_online_version(), get_local_version())?;
        if local_version != online_version {
            return Ok(Some(online_version));
        }

        // The check succeeded. Update modification time so we do not keep bugging the database.
        std::fs::File::open(CARD_INFO_LOCAL)?.set_modified(SystemTime::now())?;
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
    BufReader::new(File::open(CARD_INFO_LOCAL).await?)
        .read_line(&mut tmp)
        .await?;
    tmp.truncate(tmp.len() - 1); // Remove trailing newline
    Ok(tmp)
}
