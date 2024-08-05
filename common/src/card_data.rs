use std::ops::Index;

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::{
    card::{Card, CardPassword},
    Cards,
};

/// Internal id for cards.
///
/// The mapping will change between builds, so it should not be used for storage.
///
/// Uses [`u16`], as it can comfortably map the current amount of cards (~13 000).
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(u16);

impl Id {
    #[must_use]
    pub const fn new(id: u16) -> Self {
        Self(id)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CardData {
    cards: Vec<Card>,
    passwords: FxHashMap<u32, Id>,
}

impl CardData {
    #[must_use]
    pub fn new(cards: Cards) -> Self {
        let passwords = cards
            .iter()
            .enumerate()
            .flat_map(|(id, card)| {
                let id = Id::new(id.try_into().unwrap(/* Too many cards */));
                card.passwords.iter().map(move |password| (*password, id))
            })
            .collect();
        Self { cards, passwords }
    }

    pub fn entries(&self) -> impl Iterator<Item = (Id, &Card)> {
        self.cards
            .iter()
            .enumerate()
            .map(|(id, card)| (Id::new(id.try_into().unwrap()), card))
    }

    #[must_use]
    pub fn id_for_password(&self, password: CardPassword) -> Option<Id> {
        self.passwords.get(&password).copied()
    }
}

impl Index<Id> for CardData {
    type Output = Card;

    fn index(&self, index: Id) -> &Self::Output {
        &self.cards[usize::from(index.0)]
    }
}
