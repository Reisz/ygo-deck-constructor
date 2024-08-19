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
    fn for_part(self, part: DeckPart, cards: &CardData) -> impl Iterator<Item = (Id, u8)>;
}

impl<I: Iterator<Item = DeckEntry>> EntriesForPart for I {
    fn for_part(self, part: DeckPart, cards: &CardData) -> impl Iterator<Item = (Id, u8)> {
        self.map(move |entry| (entry.id(), entry.count(part.into())))
            .filter(move |(id, count)| *count > 0 && part.can_contain(&cards[*id]))
    }
}

#[cfg(test)]
mod test {
    use crate::{
        card::{
            test_util::{make_card, make_extra_deck_card},
            CardPassword,
        },
        card_data::CardDataStorage,
        deck::{Deck, PartType},
    };

    use super::*;

    #[test]
    fn entries_for_part() {
        const MAIN_PASSWD: CardPassword = 1234;
        const EXTRA_PASSWD: CardPassword = 2345;

        const MAIN_ID: Id = Id::new(0);
        const EXTRA_ID: Id = Id::new(1);

        let data = {
            let cards = vec![make_card(MAIN_PASSWD), make_extra_deck_card(EXTRA_PASSWD)];
            Box::leak(Box::new(CardData::from(CardDataStorage::new(cards))))
        };

        let mut deck = Deck::default();
        deck.increment(MAIN_ID, PartType::Playing, 2);
        deck.increment(MAIN_ID, PartType::Side, 3);
        deck.increment(EXTRA_ID, PartType::Playing, 4);
        deck.increment(EXTRA_ID, PartType::Side, 5);

        assert_eq!(
            deck.entries()
                .for_part(DeckPart::Main, data)
                .collect::<Vec<_>>(),
            &[(MAIN_ID, 2)]
        );
        assert_eq!(
            deck.entries()
                .for_part(DeckPart::Extra, data)
                .collect::<Vec<_>>(),
            &[(EXTRA_ID, 4)]
        );

        let mut side_cards = deck
            .entries()
            .for_part(DeckPart::Side, data)
            .collect::<Vec<_>>();
        side_cards.sort_by_key(|(id, _)| *id);
        assert_eq!(side_cards, &[(MAIN_ID, 3), (EXTRA_ID, 5)]);
    }
}
