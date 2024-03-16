use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    path::Path,
    time::{Duration, SystemTime},
};

use anyhow::Result;

use crate::{step, ygoprodeck, CARD_INFO_LOCAL, OUTPUT_FILE};

const VERSION_CHECK_INTERVAL: Duration = Duration::from_secs(60 * 60 * 24); // 1 day

#[derive(Debug, Clone)]
pub enum CacheBehavior {
    Download { online_version: String },
    Process,
    Nothing,
}

pub async fn get_behavior() -> Result<CacheBehavior> {
    Ok(if let Some(online_version) = should_download().await? {
        CacheBehavior::Download { online_version }
    } else if should_process()? {
        CacheBehavior::Process
    } else {
        CacheBehavior::Nothing
    })
}

async fn get_online_version() -> Result<String> {
    step("Checking online database version");
    let info: ygoprodeck::VersionInfo = reqwest::get(ygoprodeck::VERSION_URL)
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(info.database_version)
}

async fn should_download() -> Result<Option<String>> {
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

fn should_process() -> Result<bool> {
    if !Path::new(OUTPUT_FILE).try_exists()? {
        return Ok(true);
    }

    let output_date = fs::metadata(OUTPUT_FILE)?.modified()?;
    let executable_date = fs::metadata(std::env::current_exe()?)?.modified()?;

    if executable_date > output_date {
        return Ok(true);
    }

    Ok(false)
}
