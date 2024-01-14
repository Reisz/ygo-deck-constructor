use std::{collections::HashMap, ops::Index};

use serde::{Deserialize, Serialize};

use crate::card::{Card, Id};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CardData {
    entries: HashMap<Id, Card>,
    alternatives: HashMap<Id, Id>,
}

impl CardData {
    #[must_use]
    pub fn new(entries: HashMap<Id, Card>, alternatives: HashMap<Id, Id>) -> Self {
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
