//! `YGOPRODeck` API v7 wrapper (See [documentation](https://ygoprodeck.com/api-guide/)).

use std::io::Read;

use anyhow::Result;
use common::card::CardPassword;
use serde::Deserialize;
use serde_enum_str::Deserialize_enum_str;

pub const VERSION_URL: &str = "https://db.ygoprodeck.com/api/v7/checkDBVer.php";
pub const URL: &str = "https://db.ygoprodeck.com/api/v7/cardinfo.php";
pub const ARTWORK_URL: &str = "https://images.ygoprodeck.com/images/cards_cropped/";

#[derive(Debug, Deserialize)]
pub struct Card {
    // All Cards
    pub id: CardPassword,
    pub name: String,
    #[serde(rename = "type")]
    pub card_type: CardType,
    // Unused: frameType
    pub desc: String,

    // Monster Cards
    pub atk: Option<u16>,
    pub def: Option<u16>,
    pub level: Option<u8>,
    pub race: Option<Race>,
    pub attribute: Option<Attribute>,

    // Spell/Trap Cards
    // Duplicate: race

    // Card Archetype
    pub archetype: Option<String>,

    // Additional Response for Pendulum Monsters
    pub scale: Option<u8>,

    // Additional Response for Link Monsters
    pub linkval: Option<u8>,
    pub linkmarkers: Option<Vec<LinkMarker>>,

    pub card_images: Vec<ImageInfo>,

    pub banlist_info: Option<BanlistInfo>,
}

#[derive(Debug, Deserialize)]
pub enum CardType {
    // Main Deck Types
    #[serde(rename = "Effect Monster")]
    EffectMonster,
    #[serde(rename = "Flip Effect Monster")]
    FlipEffectMonster,
    #[serde(rename = "Flip Tuner Effect Monster")]
    FlipTunerEffectMonster,
    #[serde(rename = "Gemini Monster")]
    GeminiMonster,
    #[serde(rename = "Normal Monster")]
    NormalMonster,
    #[serde(rename = "Normal Tuner Monster")]
    NormalTunerMonster,
    #[serde(rename = "Pendulum Effect Monster")]
    PendulumEffectMonster,
    #[serde(rename = "Pendulum Effect Ritual Monster")]
    PendulumEffectRitualMonster,
    #[serde(rename = "Pendulum Flip Effect Monster")]
    PendulumFlipEffectMonster,
    #[serde(rename = "Pendulum Normal Monster")]
    PendulumNormalMonster,
    #[serde(rename = "Pendulum Tuner Effect Monster")]
    PendulumTunerEffectMonster,
    #[serde(rename = "Ritual Effect Monster")]
    RitualEffectMonster,
    #[serde(rename = "Ritual Monster")]
    RitualMonster,
    #[serde(rename = "Spell Card")]
    SpellCard,
    #[serde(rename = "Spirit Monster")]
    SpiritMonster,
    #[serde(rename = "Toon Monster")]
    ToonMonster,
    #[serde(rename = "Trap Card")]
    TrapCard,
    #[serde(rename = "Tuner Monster")]
    TunerMonster,
    #[serde(rename = "Union Effect Monster")]
    UnionEffectMonster,

    // Extra Deck Types
    #[serde(rename = "Fusion Monster")]
    FusionMonster,
    #[serde(rename = "Link Monster")]
    LinkMonster,
    #[serde(rename = "Pendulum Effect Fusion Monster")]
    PendulumEffectFusionMonster,
    #[serde(rename = "Synchro Monster")]
    SynchroMonster,
    #[serde(rename = "Synchro Pendulum Effect Monster")]
    SynchroPendulumEffectMonster,
    #[serde(rename = "Synchro Tuner Monster")]
    SynchroTunerMonster,
    #[serde(rename = "XYZ Monster")]
    XYZMonster,
    #[serde(rename = "XYZ Pendulum Effect Monster")]
    XYZPendulumEffectMonster,

    // Other Types
    #[serde(rename = "Skill Card")]
    SkillCard,
    Token,
}

#[derive(Debug, Deserialize_enum_str)]
pub enum Race {
    // Monster Cards
    Aqua,
    Beast,
    #[serde(rename = "Beast-Warrior")]
    BeastWarrior,
    #[serde(rename = "Creator-God")]
    CreatorGod,
    Cyberse,
    Dinosaur,
    #[serde(rename = "Divine-Beast")]
    DivineBeast,
    Dragon,
    Fairy,
    Fiend,
    Fish,
    Illusion,
    Insect,
    Machine,
    Plant,
    Psychic,
    Pyro,
    Reptile,
    Rock,
    #[serde(rename = "Sea Serpent")]
    SeaSerpent,
    Spellcaster,
    Thunder,
    Warrior,
    #[serde(rename = "Winged Beast")]
    WingedBeast,
    Wyrm,
    Zombie,

    // Spell Cards
    Normal,
    Field,
    Equip,
    Continuous,
    #[serde(rename = "Quick-Play")]
    QuickPlay,
    Ritual,

    // Trap Cards
    // Duplicate: Normal
    // Duplicate: Continuous,
    Counter,

    #[serde(other)]
    Other(String),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Attribute {
    Dark,
    Earth,
    Fire,
    Light,
    Water,
    Wind,
    Divine,
}

#[derive(Debug, Deserialize)]
pub enum LinkMarker {
    Top,
    Bottom,
    Left,
    Right,
    #[serde(rename = "Bottom-Left")]
    BottomLeft,
    #[serde(rename = "Bottom-Right")]
    BottomRight,
    #[serde(rename = "Top-Left")]
    TopLeft,
    #[serde(rename = "Top-Right")]
    TopRight,
}

#[derive(Debug, Deserialize)]
pub struct BanlistInfo {
    pub ban_tcg: Option<BanStatus>,
}

#[derive(Debug, Deserialize)]
pub enum BanStatus {
    Limited,
    #[serde(rename = "Semi-Limited")]
    SemiLimited,
    Banned,
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
