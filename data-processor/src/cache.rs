use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    path::Path,
    time::{Duration, SystemTime},
};

use anyhow::Result;
use serde::Deserialize;
use tokio::io::AsyncWriteExt;

use crate::{reqwest_indicatif::ProgressReader, step, ygoprodeck, CARD_INFO_LOCAL, OUTPUT_FILE};

const VERSION_CHECK_INTERVAL: Duration = Duration::from_secs(60 * 60 * 24); // 1 day

pub enum CacheResult {
    StillValid,
    ProcessingRequired,
}

pub async fn update_card_info_cache() -> Result<CacheResult> {
    if let Some(version) = should_download().await? {
        step("Downloading cards");

        let response = reqwest::get(ygoprodeck::URL).await?.error_for_status()?;
        let mut response = tokio::io::BufReader::new(ProgressReader::from_response(response));

        let mut file = tokio::io::BufWriter::new(tokio::fs::File::create(CARD_INFO_LOCAL).await?);
        file.write_all(version.as_bytes()).await?;
        file.write_all("\n".as_bytes()).await?;

        tokio::io::copy(&mut response, &mut file).await?;
        return Ok(CacheResult::ProcessingRequired);
    }

    if !Path::new(OUTPUT_FILE).try_exists()? {
        return Ok(CacheResult::ProcessingRequired);
    }

    let output_date = fs::metadata(OUTPUT_FILE)?.modified()?;
    let executable_date = fs::metadata(std::env::current_exe()?)?.modified()?;
    if executable_date > output_date {
        return Ok(CacheResult::ProcessingRequired);
    }

    Ok(CacheResult::StillValid)
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

pub async fn should_download() -> Result<Option<String>> {
    if !Path::new(CARD_INFO_LOCAL).try_exists()? {
        return Ok(Some(get_online_version().await?));
    }

    let cache_date = fs::metadata(CARD_INFO_LOCAL)?.modified()?;
    if cache_date.elapsed()? > VERSION_CHECK_INTERVAL {
        // Reset interval
        File::open(CARD_INFO_LOCAL)?.set_modified(SystemTime::now())?;

        let online_version = get_online_version().await?;
        let local_version = {
            let mut tmp = String::new();
            BufReader::new(File::open(CARD_INFO_LOCAL)?).read_line(&mut tmp)?;
            tmp.truncate(tmp.len() - 1); // Remove trailing newline
            tmp
        };

        if local_version != online_version {
            return Ok(Some(online_version));
        }
    }

    Ok(None)
}
