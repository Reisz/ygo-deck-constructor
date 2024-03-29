use std::fmt::{self, Display};

use common::card::Card;

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

    #[must_use]
    pub fn min(self) -> u8 {
        match self {
            Self::Main => 40,
            Self::Extra | Self::Side => 0,
        }
    }

    #[must_use]
    pub fn max(self) -> u8 {
        match self {
            Self::Main => 60,
            Self::Extra | Self::Side => 15,
        }
    }

    #[must_use]
    pub fn can_contain(self, card: &Card) -> bool {
        let is_extra = card.card_type.is_extra_deck_monster();

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
