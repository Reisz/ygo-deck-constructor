use crate::{card_data::Id, deck_part::DeckPart};

/// The two types of deck part a card can be in
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartType {
    /// Main or Extra deck (depends on card)
    Playing,
    /// Side deck
    Side,
}

impl From<DeckPart> for PartType {
    fn from(value: DeckPart) -> Self {
        match value {
            DeckPart::Main | DeckPart::Extra => Self::Playing,
            DeckPart::Side => Self::Side,
        }
    }
}

impl PartType {
    fn idx(self) -> usize {
        match self {
            Self::Playing => 0,
            Self::Side => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeckEntry {
    /// The card id of this entry
    id: Id,
    /// Counts for the two part types
    counts: [u8; 2],
}

impl DeckEntry {
    pub fn new(id: Id) -> Self {
        Self { id, counts: [0; 2] }
    }

    #[must_use]
    pub fn id(&self) -> Id {
        self.id
    }

    #[must_use]
    pub fn count(&self, part_type: PartType) -> usize {
        self.counts[part_type.idx()].into()
    }

    pub fn set_count(&mut self, part_type: PartType, count: u8) {
        self.counts[part_type.idx()] = count
    }
}

#[derive(Debug, Default, Clone)]
pub struct Deck(Vec<DeckEntry>);

impl Deck {
    pub fn new(mut entries: Vec<DeckEntry>) -> Self {
        entries.sort_unstable_by_key(DeckEntry::id);
        Self(entries)
    }

    pub fn increment(&mut self, id: Id, part_type: PartType, amount: u8) -> u8 {
        let idx = self
            .0
            .binary_search_by_key(&id, DeckEntry::id)
            .unwrap_or_else(|idx| {
                self.0.insert(idx, DeckEntry::new(id));
                idx
            });

        let entry = &mut self.0[idx].counts[part_type.idx()];

        if let Some(new_val) = entry.checked_add(amount) {
            *entry = new_val;
            return amount;
        }

        let ret = u8::MAX - *entry;
        *entry = u8::MAX;
        ret
    }

    pub fn decrement(&mut self, id: Id, part_type: PartType, amount: u8) -> u8 {
        if let Ok(idx) = self.0.binary_search_by_key(&id, DeckEntry::id) {
            let entry = &mut self.0[idx].counts[part_type.idx()];

            let ret = if let Some(new_val) = entry.checked_sub(amount) {
                *entry = new_val;
                amount
            } else {
                let ret = *entry;
                *entry = 0;
                ret
            };

            if self.0[idx].counts.iter().all(|count| *count == 0) {
                self.0.remove(idx);
            }

            ret
        } else {
            0
        }
    }

    pub fn entries(&self) -> impl Iterator<Item = &DeckEntry> {
        self.0.iter()
    }
}
