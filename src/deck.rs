use std::{
    cmp::Ordering,
    fmt::Display,
    ops::{Deref, Index},
};

use common::card::{Card, CardData, CardType, Id, MonsterStats, MonsterType};
use leptos::{create_rw_signal, RwSignal};

/// The three parts of a Yu-Gi-Oh deck.
#[derive(Debug, Clone, Copy)]
pub enum PartType {
    Main,
    Extra,
    Side,
}

impl PartType {
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

    pub fn max(self) -> u8 {
        match self {
            Self::Main => 60,
            Self::Extra | Self::Side => 15,
        }
    }
}

impl Display for PartType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Main => "Main",
            Self::Extra => "Extra",
            Self::Side => "Side",
        };
        write!(f, "{name}")
    }
}

type DeckOrdering = fn(&CardData, Id, Id) -> Ordering;

#[derive(Debug, Clone)]
pub struct DeckPart {
    data: Vec<(Id, usize)>,
    order: DeckOrdering,
}

impl DeckPart {
    fn new(order: DeckOrdering) -> Self {
        Self {
            data: Vec::default(),
            order,
        }
    }

    fn binary_search(&self, card_data: &CardData, id: Id) -> Result<usize, usize> {
        self.data
            .binary_search_by(|(probe, _)| (self.order)(card_data, *probe, id))
    }

    pub fn add(&mut self, card_data: &CardData, id: Id) {
        match self.binary_search(card_data, id) {
            Ok(pos) => self.data[pos].1 += 1,
            Err(pos) => self.data.insert(pos, (id, 1)),
        }
    }

    pub fn remove(&mut self, card_data: &CardData, id: Id) {
        if let Ok(pos) = self.binary_search(card_data, id) {
            if self.data[pos].1 == 1 {
                self.data.remove(pos);
            } else {
                self.data[pos].1 -= 1;
            }
        }
    }

    pub fn len(&self) -> usize {
        self.data.iter().map(|(_, count)| count).sum()
    }
}

impl Deref for DeckPart {
    type Target = Vec<(Id, usize)>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Deck {
    parts: [RwSignal<DeckPart>; 3],
}

impl Deck {
    pub fn new(order: DeckOrdering) -> Self {
        Self {
            parts: [
                create_rw_signal(DeckPart::new(order)),
                create_rw_signal(DeckPart::new(order)),
                create_rw_signal(DeckPart::new(order)),
            ],
        }
    }
}

impl Index<PartType> for Deck {
    type Output = RwSignal<DeckPart>;

    fn index(&self, index: PartType) -> &Self::Output {
        &self.parts[index as usize]
    }
}
