use serde::{Deserialize, Serialize};

/// Full card data after extraction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FullCard {
    pub name: String,
    pub main_password: CardPassword,
    pub all_passwords: Vec<CardPassword>,
    pub description: CardDescription,
    pub search_text: String,
    pub card_type: CardType,
    pub limit: CardLimit,
}

/// Card data used in the app.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    pub name: &'static str,
    pub password: CardPassword,
    pub description: CardDescription,
    pub search_text: &'static str,
    pub card_type: CardType,
    pub limit: CardLimit,
}

/// Type used for [Passwords](https://yugipedia.com/wiki/Password).
///
/// Uses [`u32`] as it is the smallest integer type which can fit all eight-digit numbers.
pub type CardPassword = u32;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum CardDescription {
    Regular(Vec<CardDescriptionPart>),
    Pendulum {
        spell_effect: Vec<CardDescriptionPart>,
        monster_effect: Vec<CardDescriptionPart>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum CardDescriptionPart {
    Paragraph(String),
    List(Vec<String>),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum CardType {
    Monster {
        race: Race,
        attribute: Attribute,
        // See https://yugipedia.com/wiki/Monster_Card#Classifications
        stats: MonsterStats,
        effect: MonsterEffect,
        is_tuner: bool,
    },
    Spell(SpellType),
    Trap(TrapType),
}

impl CardType {
    #[must_use]
    pub fn is_extra_deck_monster(&self) -> bool {
        matches!(
            self,
            CardType::Monster {
                stats: MonsterStats::Normal {
                    monster_type: Some(
                        MonsterType::Fusion | MonsterType::Synchro | MonsterType::Xyz
                    ),
                    ..
                } | MonsterStats::Link { .. },
                ..
            }
        )
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub enum Race {
    Aqua,
    Beast,
    BeastWarrior,
    CreatorGod,
    Cyberse,
    Dinosaur,
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
    SeaSerpent,
    Spellcaster,
    Thunder,
    Warrior,
    WingedBeast,
    Wyrm,
    Zombie,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub enum Attribute {
    Dark,
    Earth,
    Fire,
    Light,
    Water,
    Wind,
    Divine,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum MonsterStats {
    Normal {
        atk: u16,
        def: u16,
        level: u8,
        monster_type: Option<MonsterType>,
        pendulum_scale: Option<u8>,
    },
    Link {
        atk: u16,
        link_value: u8,
        link_markers: LinkMarkers,
    },
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub enum MonsterType {
    Fusion,
    Ritual,
    Synchro,
    Xyz,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub enum MonsterEffect {
    Normal,
    Effect,
    Spirit,
    Toon,
    Union,
    Gemini,
    Flip,
}

#[derive(Default, Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct LinkMarkers(u8);

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkMarker {
    TopLeft,
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
}

impl LinkMarker {
    pub fn iter() -> impl Iterator<Item = Self> {
        [
            LinkMarker::TopLeft,
            LinkMarker::Top,
            LinkMarker::TopRight,
            LinkMarker::Right,
            LinkMarker::BottomRight,
            LinkMarker::Bottom,
            LinkMarker::BottomLeft,
            LinkMarker::Left,
        ]
        .into_iter()
    }
}

impl LinkMarkers {
    pub fn add(&mut self, marker: LinkMarker) {
        let idx = marker as u8;
        self.0 |= 1 << idx;
    }

    #[must_use]
    pub fn has(&self, marker: LinkMarker) -> bool {
        let idx = marker as u8;
        (self.0 & (1 << idx)) >> idx == 1
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub enum SpellType {
    Normal,
    Field,
    Equip,
    Continuous,
    QuickPlay,
    Ritual,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub enum TrapType {
    Normal,
    Continuous,
    Counter,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub enum CardLimit {
    Unlimited,
    SemiLimited,
    Limited,
    Banned,
}

impl CardLimit {
    #[must_use]
    pub fn count(self) -> u8 {
        match self {
            Self::Unlimited => 3,
            Self::SemiLimited => 2,
            Self::Limited => 1,
            Self::Banned => 0,
        }
    }
}

pub mod test_util {
    use super::*;

    pub fn make_card(password: CardPassword) -> FullCard {
        FullCard {
            name: String::new(),
            main_password: password,
            all_passwords: vec![password],
            description: CardDescription::Regular(vec![]),
            search_text: String::new(),
            card_type: CardType::Spell(SpellType::Normal),
            limit: CardLimit::Unlimited,
        }
    }

    pub fn make_extra_deck_card(password: CardPassword) -> FullCard {
        FullCard {
            name: String::new(),
            main_password: password,
            all_passwords: vec![password],
            description: CardDescription::Regular(vec![]),
            search_text: String::new(),
            card_type: CardType::Monster {
                race: Race::Aqua,
                attribute: Attribute::Dark,
                stats: MonsterStats::Normal {
                    atk: 0,
                    def: 0,
                    level: 0,
                    monster_type: Some(MonsterType::Fusion),
                    pendulum_scale: None,
                },
                effect: MonsterEffect::Normal,
                is_tuner: false,
            },
            limit: CardLimit::Unlimited,
        }
    }
}
