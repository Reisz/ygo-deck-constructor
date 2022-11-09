use std::io::Read;

use anyhow::Result;
use serde::Deserialize;

pub const URL: &str = "https://db.ygoprodeck.com/api/v7/cardinfo.php";

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

pub fn parse<R: Read>(reader: R) -> Result<Vec<Card>> {
    let result: Wrapper = serde_json::from_reader(reader)?;
    Ok(result.data)
}
