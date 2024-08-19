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
    pub fn count(&self, part_type: PartType) -> u8 {
        self.counts[part_type.idx()]
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

    pub fn entries(&self) -> impl Iterator<Item = DeckEntry> + '_ {
        self.0.iter().copied()
    }
}

pub mod test_util {
    #[macro_export]
    macro_rules! assert_part_eq {
        ($deck: expr, $part_type: expr, $expected: expr) => {
            assert_eq!(
                $deck
                    .entries()
                    .map(|entry| (entry.id(), entry.count($part_type)))
                    .filter(|(_, count)| *count > 0)
                    .collect::<Vec<_>>(),
                $expected.to_vec(),
                "part_type = {:?}",
                $part_type
            );
        };
    }
}

#[cfg(test)]
mod test {
    use crate::assert_part_eq;

    use super::*;

    fn part_types() -> impl Iterator<Item = PartType> {
        [PartType::Playing, PartType::Side].into_iter()
    }

    fn other(part_type: PartType) -> PartType {
        match part_type {
            PartType::Playing => PartType::Side,
            PartType::Side => PartType::Playing,
        }
    }

    #[test]
    fn entry_memory_usage() {
        assert!(size_of::<DeckEntry>() <= 4);
        assert!(align_of::<DeckEntry>() <= 4);
    }

    #[test]
    fn empty_deck() {
        let deck = Deck::default();

        assert_part_eq!(&deck, PartType::Playing, &[]);
        assert_part_eq!(&deck, PartType::Side, &[]);
    }

    #[test]
    fn add_remove() {
        const ID: Id = Id::new(1234);
        const AMOUNT: u8 = 43;

        for part in part_types() {
            let mut deck = Deck::default();

            deck.increment(ID, part, AMOUNT);
            assert_part_eq!(&deck, part, &[(ID, AMOUNT)]);
            assert_part_eq!(&deck, other(part), &[]);

            deck.decrement(ID, part, AMOUNT);
            assert_part_eq!(&deck, part, &[]);
            assert_part_eq!(&deck, other(part), &[]);
        }
    }

    #[test]
    fn remove_on_empty() {
        const ID: Id = Id::new(1234);
        const AMOUNT: u8 = 43;

        let mut deck = Deck::default();
        deck.decrement(ID, PartType::Playing, AMOUNT);
        deck.decrement(ID, PartType::Side, AMOUNT);

        assert_part_eq!(&deck, PartType::Playing, &[]);
        assert_part_eq!(&deck, PartType::Side, &[]);
    }

    #[test]
    fn remove_too_many() {
        const ID: Id = Id::new(1234);
        const AMOUNT: u8 = 43;
        const REMOVE_AMOUNT: u8 = 98;

        for part in part_types() {
            let mut deck = Deck::default();
            deck.increment(ID, part, AMOUNT);
            deck.decrement(ID, part, REMOVE_AMOUNT);

            assert_part_eq!(&deck, part, &[]);
            assert_part_eq!(&deck, other(part), &[]);
        }
    }

    #[test]
    fn add_too_many() {
        const ID: Id = Id::new(1234);
        const AMOUNT: u8 = 43;

        for part in part_types() {
            let mut deck = Deck::default();
            deck.increment(ID, part, u8::MAX - 1);
            deck.increment(ID, part, AMOUNT);

            assert_part_eq!(&deck, part, &[(ID, u8::MAX)]);
            assert_part_eq!(&deck, other(part), &[]);
        }
    }
}
