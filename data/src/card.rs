use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Id(u64);

impl Id {
    #[must_use]
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    #[must_use]
    pub fn get(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Card {
    pub name: String,
    pub desc: String,
}
