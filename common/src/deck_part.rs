use std::fmt::{self, Display};

use crate::{
    card::Card,
    card_data::{CardData, Id},
    deck::DeckEntry,
};

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

pub trait EntriesForPart {
    fn for_part(self, part: DeckPart, cards: &CardData) -> impl Iterator<Item = (Id, usize)>;
}

impl<I: Iterator<Item = DeckEntry>> EntriesForPart for I {
    fn for_part(self, part: DeckPart, cards: &CardData) -> impl Iterator<Item = (Id, usize)> {
        self.map(move |entry| (entry.id(), entry.count(part.into())))
            .filter(move |(id, count)| *count > 0 && part.can_contain(&cards[*id]))
    }
}
