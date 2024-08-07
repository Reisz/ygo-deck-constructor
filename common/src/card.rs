use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Card {
    pub name: String,
    pub passwords: Vec<CardPassword>,
    pub description: CardDescription,
    pub search_text: String,
    pub card_type: CardType,
    pub limit: CardLimit,
    pub archetype: Option<String>,
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
    pub fn is_monster(&self) -> bool {
        matches!(self, Self::Monster { .. })
    }

    #[must_use]
    pub fn is_spell(&self) -> bool {
        matches!(self, Self::Spell { .. })
    }

    #[must_use]
    pub fn is_trap(&self) -> bool {
        matches!(self, Self::Trap { .. })
    }

    #[must_use]
    pub fn is_pendulum_monster(&self) -> bool {
        if let Self::Monster { stats, .. } = self {
            stats.is_pendulum()
        } else {
            false
        }
    }

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

impl MonsterStats {
    #[must_use]
    pub fn is_pendulum(&self) -> bool {
        matches!(
            self,
            Self::Normal {
                pendulum_scale: Some(_),
                ..
            }
        )
    }
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
        self.0[marker as usize] = true;
    }

    #[must_use]
    pub fn has(&self, marker: LinkMarker) -> bool {
        self.0[marker as usize]
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
