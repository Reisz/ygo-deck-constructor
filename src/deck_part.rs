use std::fmt::{self, Display};

use common::card::{Card, CardType, MonsterStats, MonsterType};

#[derive(Debug, Clone, Copy)]
pub enum DeckPart {
    Main,
    Extra,
    Side,
}

impl DeckPart {
    pub fn iter() -> impl Iterator<Item = Self> {
        [Self::Main, Self::Extra, Self::Side].into_iter()
    }

    pub fn min(self) -> u8 {
        match self {
            Self::Main => 40,
            Self::Extra | Self::Side => 0,
        }
    }

    pub fn max(self) -> u8 {
        match self {
            Self::Main => 60,
            Self::Extra | Self::Side => 15,
        }
    }

    pub fn can_contain(self, card: &Card) -> bool {
        let is_extra = matches!(
            card.card_type,
            CardType::Monster {
                stats: MonsterStats::Normal {
                    monster_type: Some(
                        MonsterType::Fusion | MonsterType::Synchro | MonsterType::Xyz
                    ),
                    ..
                } | MonsterStats::Link { .. },
                ..
            }
        );

        match self {
            Self::Main => !is_extra,
            Self::Extra => is_extra,
            Self::Side => true,
        }
    }
}

impl Display for DeckPart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Main => "Main",
            Self::Extra => "Extra",
            Self::Side => "Side",
        };

        write!(f, "{name}")
    }
}
