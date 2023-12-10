use common::card::{CardData, Id};

use crate::deck_part::DeckPart;

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

/// Counter with saturating value, which returns the actual change for repeatable message-based undo-redo.
#[derive(Debug, Clone, Copy)]
struct ReversibleSaturatingCounter(u32);

impl ReversibleSaturatingCounter {
    fn increment(&mut self, amount: u32) -> u32 {
        if let Some(new_val) = self.0.checked_add(amount) {
            self.0 = new_val;
            return amount;
        }

        let ret = u32::MAX - self.0;
        self.0 = u32::MAX;
        ret
    }

    fn decrement(&mut self, amount: u32) -> u32 {
        if let Some(new_val) = self.0.checked_sub(amount) {
            self.0 = new_val;
            return amount;
        }

        let ret = u32::MIN + self.0;
        self.0 = u32::MIN;
        ret
    }

    fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DeckEntry {
    /// The card id of this entry
    id: Id,
    /// Counts for the two part types
    counts: [ReversibleSaturatingCounter; 2],
}

impl DeckEntry {
    fn new(id: Id) -> Self {
        Self {
            id,
            counts: [ReversibleSaturatingCounter(0); 2],
        }
    }

    #[must_use]
    pub fn id(&self) -> Id {
        self.id
    }

    fn idx(part_type: PartType) -> usize {
        match part_type {
            PartType::Playing => 0,
            PartType::Side => 1,
        }
    }

    fn count_mut(&mut self, part_type: PartType) -> &mut ReversibleSaturatingCounter {
        &mut self.counts[Self::idx(part_type)]
    }

    #[must_use]
    pub fn count(&self, part_type: PartType) -> usize {
        self.counts[Self::idx(part_type)].get().try_into().unwrap()
    }

    fn empty(&self) -> bool {
        self.counts[0].get() == 0 && self.counts[1].get() == 0
    }
}

#[derive(Debug, Default, Clone)]
pub struct Deck(Vec<DeckEntry>);

impl Deck {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment(&mut self, id: Id, part_type: PartType, amount: u32) -> u32 {
        let idx = self
            .0
            .binary_search_by_key(&id, DeckEntry::id)
            .unwrap_or_else(|idx| {
                self.0.insert(idx, DeckEntry::new(id));
                idx
            });
        self.0[idx].count_mut(part_type).increment(amount)
    }

    pub fn decrement(&mut self, id: Id, part_type: PartType, amount: u32) -> u32 {
        if let Ok(idx) = self.0.binary_search_by_key(&id, DeckEntry::id) {
            let ret = self.0[idx].count_mut(part_type).decrement(amount);
            if self.0[idx].empty() {
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

    pub fn iter_part(
        &self,
        cards: &'static CardData,
        part: DeckPart,
    ) -> impl Iterator<Item = (Id, usize)> + '_ {
        self.entries()
            .map(move |entry| (entry.id(), entry.count(part.into())))
            .filter(move |(id, count)| *count > 0 && part.can_contain(&cards[&id]))
    }
}

#[cfg(test)]
mod test {
    use std::{
        collections::HashMap,
        mem::{align_of, size_of},
    };

    use common::card::{Card, MonsterType};

    use super::*;

    fn assert_part_eq(deck: &Deck, part_type: PartType, expected: &[(Id, usize)]) {
        assert_eq!(
            deck.entries()
                .map(|entry| (entry.id(), entry.count(part_type)))
                .filter(|(_, count)| *count > 0)
                .collect::<Vec<_>>(),
            expected.to_vec(),
            "part_type = {part_type:?}"
        );
    }

    #[derive(Debug, Clone, Copy)]
    struct TestCase {
        current: PartType,
        other: PartType,
    }

    impl TestCase {
        fn iter() -> impl Iterator<Item = Self> {
            [
                Self {
                    current: PartType::Playing,
                    other: PartType::Side,
                },
                Self {
                    current: PartType::Side,
                    other: PartType::Playing,
                },
            ]
            .into_iter()
        }
    }

    #[test]
    fn entry_memory_usage() {
        assert!(size_of::<DeckEntry>() <= 16);
        assert!(align_of::<DeckEntry>() <= 16);
    }

    #[test]
    fn empty_deck() {
        let deck = Deck::new();

        assert_part_eq(&deck, PartType::Playing, &[]);
        assert_part_eq(&deck, PartType::Side, &[]);
    }

    #[test]
    fn add_remove() {
        const ID: Id = Id::new(1234);
        const AMOUNT: u32 = 4321;

        for TestCase { current, other } in TestCase::iter() {
            let mut deck = Deck::new();
            assert_eq!(deck.increment(ID, current, AMOUNT), AMOUNT);

            assert_part_eq(&deck, current, &[(ID, AMOUNT as usize)]);
            assert_part_eq(&deck, other, &[]);

            assert_eq!(deck.decrement(ID, current, AMOUNT), AMOUNT);
            assert_part_eq(&deck, current, &[]);
            assert_part_eq(&deck, other, &[]);
        }
    }

    #[test]
    fn remove_on_empty() {
        const ID: Id = Id::new(1234);
        const AMOUNT: u32 = 4321;

        let mut deck = Deck::new();
        assert_eq!(deck.decrement(ID, PartType::Playing, AMOUNT), 0);
        assert_eq!(deck.decrement(ID, PartType::Side, AMOUNT), 0);

        assert_part_eq(&deck, PartType::Playing, &[]);
        assert_part_eq(&deck, PartType::Side, &[]);
    }

    #[test]
    fn remove_too_many() {
        const ID: Id = Id::new(1234);
        const AMOUNT: u32 = 4321;
        const REMOVE_AMOUNT: u32 = 9876;

        for TestCase { current, other: _ } in TestCase::iter() {
            let mut deck = Deck::new();
            assert_eq!(deck.increment(ID, current, AMOUNT), AMOUNT);
            assert_eq!(deck.decrement(ID, current, REMOVE_AMOUNT), AMOUNT);

            assert_part_eq(&deck, PartType::Playing, &[]);
            assert_part_eq(&deck, PartType::Side, &[]);
        }
    }

    #[test]
    fn add_too_many() {
        const ID: Id = Id::new(1234);
        const AMOUNT: u32 = 4321;

        for TestCase { current, other } in TestCase::iter() {
            let mut deck = Deck::new();
            assert_eq!(deck.increment(ID, current, u32::MAX - 1), u32::MAX - 1);
            assert_eq!(deck.increment(ID, current, AMOUNT), 1);

            assert_part_eq(&deck, current, &[(ID, u32::MAX as usize)]);
            assert_part_eq(&deck, other, &[]);
        }
    }

    #[test]
    fn iter_part() {
        const MAIN_ID: Id = Id::new(1234);
        const EXTRA_ID: Id = Id::new(2345);

        let data = {
            let mut data = Box::<HashMap<Id, Card>>::default();

            data.insert(
                MAIN_ID,
                Card {
                    name: String::new(),
                    description: String::new(),
                    card_type: common::card::CardType::Spell(common::card::SpellType::Normal),
                    limit: common::card::CardLimit::Unlimited,
                    archetype: None,
                },
            );

            data.insert(
                EXTRA_ID,
                Card {
                    name: String::new(),
                    description: String::new(),
                    card_type: common::card::CardType::Monster {
                        race: common::card::Race::Aqua,
                        attribute: common::card::Attribute::Dark,
                        stats: common::card::MonsterStats::Normal {
                            atk: 0,
                            def: 0,
                            level: 0,
                            monster_type: Some(MonsterType::Fusion),
                            pendulum_scale: None,
                        },
                        effect: common::card::MonsterEffect::Normal,
                        is_tuner: false,
                    },
                    limit: common::card::CardLimit::Unlimited,
                    archetype: None,
                },
            );

            Box::leak(data)
        };

        let mut deck = Deck::new();
        deck.increment(MAIN_ID, PartType::Playing, 2);
        deck.increment(MAIN_ID, PartType::Side, 3);
        deck.increment(EXTRA_ID, PartType::Playing, 4);
        deck.increment(EXTRA_ID, PartType::Side, 5);

        assert_eq!(
            deck.iter_part(data, DeckPart::Main).collect::<Vec<_>>(),
            &[(MAIN_ID, 2)]
        );
        assert_eq!(
            deck.iter_part(data, DeckPart::Extra).collect::<Vec<_>>(),
            &[(EXTRA_ID, 4)]
        );

        let mut side_cards = deck.iter_part(data, DeckPart::Side).collect::<Vec<_>>();
        side_cards.sort_by_key(|(id, _)| id.get());
        assert_eq!(side_cards, &[(MAIN_ID, 3), (EXTRA_ID, 5)]);
    }
}
