use std::ops::Index;

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::card::{Card, CardDescription, CardLimit, CardPassword, CardType, FullCard};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardStorage {
    pub name: String,
    pub password: CardPassword,
    pub description: CardDescription,
    pub search_text: String,
    pub card_type: CardType,
    pub limit: CardLimit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardDataStorage {
    cards: Vec<CardStorage>,
    passwords: FxHashMap<CardPassword, Id>,
}

impl CardDataStorage {
    pub fn new(cards: Vec<FullCard>) -> Self {
        let passwords = cards
            .iter()
            .enumerate()
            .flat_map(|(id, card)| {
                let id = Id::new(id.try_into().unwrap(/* Too many cards */));
                card.all_passwords
                    .iter()
                    .map(move |password| (*password, id))
            })
            .collect();
        let cards = cards
            .into_iter()
            .map(|card| CardStorage {
                name: card.name,
                password: card.main_password,
                description: card.description,
                search_text: card.search_text,
                card_type: card.card_type,
                limit: card.limit,
            })
            .collect();
        Self { cards, passwords }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CardData {
    cards: &'static [Card],
    passwords: &'static FxHashMap<CardPassword, Id>,
}

impl CardData {
    pub fn get(self, id: Id) -> &'static Card {
        &self.cards[usize::from(id.0)]
    }

    pub fn entries(self) -> impl Iterator<Item = (Id, &'static Card)> {
        self.cards
            .iter()
            .enumerate()
            .map(|(id, card)| (Id::new(id.try_into().unwrap()), card))
    }

    #[must_use]
    pub fn id_for_password(self, password: CardPassword) -> Option<Id> {
        self.passwords.get(&password).copied()
    }
}

impl From<CardDataStorage> for CardData {
    fn from(value: CardDataStorage) -> Self {
        let cards = value
            .cards
            .into_iter()
            .map(|card| Card {
                name: Box::leak(card.name.into_boxed_str()),
                password: card.password,
                description: card.description,
                search_text: Box::leak(card.search_text.into_boxed_str()),
                card_type: card.card_type,
                limit: card.limit,
            })
            .collect();
        Self {
            cards: Box::leak(cards),
            passwords: Box::leak(Box::new(value.passwords)),
        }
    }
}

impl Index<Id> for CardData {
    type Output = Card;

    fn index(&self, index: Id) -> &Self::Output {
        self.get(index)
    }
}
