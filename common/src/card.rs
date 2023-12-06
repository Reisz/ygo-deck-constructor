use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(u64);

impl Id {
    #[must_use]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    #[must_use]
    pub const fn get(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Card {
    pub name: String,
    pub description: String,
    pub card_type: CardType,
    pub limit: CardLimit,
    pub archetype: Option<String>,
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
pub struct LinkMarkers([bool; 8]);

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

impl LinkMarkers {
    pub fn add(&mut self, marker: LinkMarker) {
        self.0[marker as usize] = true;
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

pub type CardData = HashMap<Id, Card>;
