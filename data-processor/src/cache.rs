use std::{
    path::Path,
    time::{Duration, SystemTime},
};

use anyhow::Result;
use serde::Deserialize;
use tokio::{
    fs::{self, File},
    io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader, BufWriter},
    try_join,
};

use crate::{reqwest_indicatif::ProgressReader, step, ygoprodeck, CARD_INFO_LOCAL, OUTPUT_FILE};

const VERSION_CHECK_INTERVAL: Duration = Duration::from_secs(60 * 60 * 24); // 1 day

pub enum CacheResult {
    StillValid,
    ProcessingRequired,
}

pub async fn update_card_info_cache() -> Result<CacheResult> {
    if let Some(version) = should_download().await? {
        step("Downloading cards");
        let (mut response, mut file) = try_join!(
            create_database_download(),
            create_database_cache_writer(&version)
        )?;
        tokio::io::copy(&mut response, &mut file).await?;
        return Ok(CacheResult::ProcessingRequired);
    }

    if !Path::new(OUTPUT_FILE).try_exists()? {
        return Ok(CacheResult::ProcessingRequired);
    }

    let (output_date, executable_date) = try_join!(
        get_modification_time(OUTPUT_FILE),
        get_modification_time(std::env::current_exe()?)
    )?;
    if executable_date > output_date {
        return Ok(CacheResult::ProcessingRequired);
    }

    Ok(CacheResult::StillValid)
}

async fn create_database_download() -> Result<impl AsyncBufRead> {
    let response = reqwest::get(ygoprodeck::URL).await?.error_for_status()?;
    Ok(BufReader::new(ProgressReader::from_response(response)))
}

async fn create_database_cache_writer(version: &str) -> Result<impl AsyncWrite> {
    let mut file = BufWriter::new(File::create(CARD_INFO_LOCAL).await?);
    file.write_all(version.as_bytes()).await?;
    file.write_all("\n".as_bytes()).await?;
    Ok(file)
}

async fn get_modification_time(path: impl AsRef<Path>) -> Result<SystemTime> {
    Ok(fs::metadata(path).await?.modified()?)
}

pub async fn should_download() -> Result<Option<String>> {
    if !Path::new(CARD_INFO_LOCAL).try_exists()? {
        return Ok(Some(get_online_version().await?));
    }

    let cache_date = fs::metadata(CARD_INFO_LOCAL).await?.modified()?;
    if cache_date.elapsed()? > VERSION_CHECK_INTERVAL {
        // Reset interval
        std::fs::File::open(CARD_INFO_LOCAL)?.set_modified(SystemTime::now())?;

        let (online_version, local_version) = try_join!(get_online_version(), get_local_version())?;
        if local_version != online_version {
            return Ok(Some(online_version));
        }
    }

    Ok(None)
}

async fn get_online_version() -> Result<String> {
    pub const VERSION_URL: &str = "https://db.ygoprodeck.com/api/v7/checkDBVer.php";

    #[derive(Debug, Deserialize)]
    pub struct VersionInfo {
        pub database_version: String,
    }

    step("Checking online database version");
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
