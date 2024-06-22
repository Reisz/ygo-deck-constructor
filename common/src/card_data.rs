use std::{collections::HashMap, iter, ops::Index};

use serde::{Deserialize, Serialize};

use crate::{
    card::{Card, Id},
    Cards,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CardData {
    entries: HashMap<Id, Card>,
    alternatives: HashMap<Id, Id>,
}

impl CardData {
    #[must_use]
    pub fn new(cards: Cards) -> Self {
        let mut alternatives = HashMap::new();
        let entries = cards
            .into_iter()
            .map(|card| {
                let id = *card.ids.first().unwrap();
                alternatives.extend(card.ids.iter().copied().zip(iter::repeat(id)));
                (id, card)
            })
            .collect();
        Self {
            entries,
            alternatives,
        }
    }

    #[must_use]
    pub fn entries(&self) -> &HashMap<Id, Card> {
        &self.entries
    }

    #[must_use]
    pub fn contains(&self, id: Id) -> bool {
        self.entries.contains_key(&id)
    }

    #[must_use]
    pub fn normalize(&self, id: Id) -> Id {
        self.alternatives.get(&id).copied().unwrap_or(id)
    }
}

impl Index<Id> for CardData {
    type Output = Card;

    fn index(&self, index: Id) -> &Self::Output {
        &self.entries[&index]
    }
}
