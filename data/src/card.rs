use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Card {
    pub id: String,
    pub name: String,
    pub desc: String,
}
