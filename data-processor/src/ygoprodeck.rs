use std::io::Read;

use anyhow::Result;
use serde::Deserialize;

pub const VERSION_URL: &str = "https://db.ygoprodeck.com/api/v7/checkDBVer.php";
pub const URL: &str = "https://db.ygoprodeck.com/api/v7/cardinfo.php";

#[derive(Debug, Deserialize)]
struct VersionInfo {
    pub database_version: String,
}

#[derive(Debug, Deserialize)]
pub struct Card {
    pub id: u64,
    pub name: String,
    pub desc: String,
}

#[derive(Debug, Deserialize)]
struct Wrapper {
    data: Vec<Card>,
}

pub fn get_version<R: Read>(reader: R) -> Result<String> {
    let [result]: [VersionInfo; 1] = serde_json::from_reader(reader)?;
    Ok(result.database_version)
}

pub fn parse<R: Read>(reader: R) -> Result<Vec<Card>> {
    let result: Wrapper = serde_json::from_reader(reader)?;
    Ok(result.data)
}
