//! `YGOPRODeck` API v7 wrapper (See [documentation](https://ygoprodeck.com/api-guide/)).

use std::io::Read;

use anyhow::Result;
use common::card::CardPassword;
use serde::Deserialize;

pub const VERSION_URL: &str = "https://db.ygoprodeck.com/api/v7/checkDBVer.php";
pub const URL: &str = "https://db.ygoprodeck.com/api/v7/cardinfo.php";
pub const ARTWORK_URL: &str = "https://images.ygoprodeck.com/images/cards_cropped/";

#[derive(Debug, Deserialize)]
pub struct Card {
    // All Cards
    pub id: CardPassword,
    pub name: String,
    #[serde(rename = "type")]
    pub card_type: String,
    // Unused: frameType
    pub desc: String,

    // Monster Cards
    pub atk: Option<i16>,
    pub def: Option<i16>,
    pub level: Option<u8>,
    pub race: Option<String>,
    pub attribute: Option<String>,

    // Spell/Trap Cards
    // Duplicate: race

    // Card Archetype
    pub archetype: Option<String>,

    // Additional Response for Pendulum Monsters
    pub scale: Option<u8>,

    // Additional Response for Link Monsters
    pub linkval: Option<u8>,
    pub linkmarkers: Option<Vec<String>>,

    pub card_images: Vec<ImageInfo>,

    pub banlist_info: Option<BanlistInfo>,
}

#[derive(Debug, Deserialize)]
pub struct BanlistInfo {
    pub ban_tcg: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ImageInfo {
    pub id: CardPassword,
}

#[derive(Debug, Deserialize)]
struct Wrapper {
    data: Vec<Card>,
}

pub fn parse<R: Read>(reader: R) -> Result<Vec<Card>> {
    let result: Wrapper = serde_json::from_reader(reader)?;
    Ok(result.data)
}
