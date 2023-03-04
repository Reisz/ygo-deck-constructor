use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Id(u64);

impl Id {
    #[must_use]
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    #[must_use]
    pub fn get(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Card {
    pub name: String,
    pub desc: String,
    pub card_type: CardType,
    pub limit: CardLimit,
    pub archetype: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum CardType {
    Monster {
        race: (),      // TODO
        attribute: (), // TODO
        // See https://yugipedia.com/wiki/Monster_Card#Classifications
        stats: MonsterStats,
        effect: MonsterEffect,
        is_tuner: bool,
    },
    Spell(SpellType),
    Trap(TrapType),
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum MonsterType {
    Fusion,
    Ritual,
    Synchro,
    Xyz,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum SpellType {
    Normal,
    Field,
    Equip,
    Continuous,
    QuickPlay,
    Ritual,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum TrapType {
    Normal,
    Continuous,
    Counter,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum CardLimit {
    Unlimited,
    SemiLimited,
    Limited,
    Banned,
}

pub type CardData = HashMap<Id, Card>;
