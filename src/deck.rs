use std::fmt;

use common::{card::Id, card_data::CardData};

use crate::{
    deck_part::DeckPart,
    text_encoding::TextEncoding,
    undo_redo::{UndoRedo, UndoRedoMessage},
};

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeckEntry {
    /// The card id of this entry
    id: Id,
    /// Counts for the two part types
    counts: [ReversibleSaturatingCounter; 2],
}

impl TextEncoding for DeckEntry {
    fn encode(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        let [playing, side] = self.counts;
        write!(writer, "{}:{}:{}", self.id, playing.get(), side.get())
    }

    fn decode(text: &str) -> Option<Self> {
        let (id, text) = text.split_once(':')?;
        let (playing, side) = text.split_once(':')?;

        let id = Id::new(id.parse().ok()?);
        let playing = ReversibleSaturatingCounter(playing.parse().ok()?);
        let side = ReversibleSaturatingCounter(side.parse().ok()?);

        Some(Self {
            id,
            counts: [playing, side],
        })
    }
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

#[derive(Debug, Clone, Copy)]
enum DeckMessage {
    Inc(Id, PartType, u32),
    Dec(Id, PartType, u32),
}

impl UndoRedoMessage for DeckMessage {
    fn invert(self) -> Self {
        match self {
            Self::Inc(id, part_type, amount) => Self::Dec(id, part_type, amount),
            Self::Dec(id, part_type, amount) => Self::Inc(id, part_type, amount),
        }
    }
}

impl TextEncoding for DeckMessage {
    fn encode(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        let sign = match self {
            Self::Inc(..) => '+',
            Self::Dec(..) => '-',
        };

        let (Self::Inc(id, part, count) | Self::Dec(id, part, count)) = self;
        let part = match part {
            PartType::Playing => 'p',
            PartType::Side => 's',
        };

        write!(writer, "{sign}{part}{id}:{count}")
    }

    fn decode(text: &str) -> Option<Self> {
        text.starts_with(['+', '-']).then_some(())?;
        let (sign, text) = text.split_at(1);

        text.starts_with(['p', 's']).then_some(())?;
        let (part, text) = text.split_at(1);

        let (id, count) = text.split_once(':')?;

        let id = Id::new(id.parse().ok()?);
        dbg!("a");
        let part = match part {
            "p" => PartType::Playing,
            "s" => PartType::Side,
            _ => return None,
        };
        let count = count.parse().ok()?;
        dbg!("a");

        let result = match sign {
            "+" => Self::Inc(id, part, count),
            "-" => Self::Dec(id, part, count),
            _ => return None,
        };

        Some(result)
    }
}

#[derive(Debug, Default, Clone)]
pub struct Deck {
    entries: Vec<DeckEntry>,
    undo_redo: UndoRedo<DeckMessage>,
}

impl Deck {
    pub fn increment(&mut self, id: Id, part_type: PartType, amount: u32) {
        let amount = self.increment_internal(id, part_type, amount);
        if amount > 0 {
            self.undo_redo
                .push_action(DeckMessage::Inc(id, part_type, amount));
        }
    }

    pub fn decrement(&mut self, id: Id, part_type: PartType, amount: u32) {
        let amount = self.decrement_internal(id, part_type, amount);
        if amount > 0 {
            self.undo_redo
                .push_action(DeckMessage::Dec(id, part_type, amount));
        }
    }

    pub fn undo(&mut self) {
        if let Some(message) = self.undo_redo.undo() {
            self.apply(message);
        }
    }

    pub fn redo(&mut self) {
        if let Some(message) = self.undo_redo.redo() {
            self.apply(message);
        }
    }

    pub fn reset_history(&mut self) {
        self.undo_redo = UndoRedo::default();
    }

    fn apply(&mut self, message: DeckMessage) {
        match message {
            DeckMessage::Inc(id, part_type, amount) => {
                debug_assert_eq!(amount, self.increment_internal(id, part_type, amount));
            }
            DeckMessage::Dec(id, part_type, amount) => {
                debug_assert_eq!(amount, self.decrement_internal(id, part_type, amount));
            }
        }
    }

    fn increment_internal(&mut self, id: Id, part_type: PartType, amount: u32) -> u32 {
        let idx = self
            .entries
            .binary_search_by_key(&id, DeckEntry::id)
            .unwrap_or_else(|idx| {
                self.entries.insert(idx, DeckEntry::new(id));
                idx
            });
        self.entries[idx].count_mut(part_type).increment(amount)
    }

    fn decrement_internal(&mut self, id: Id, part_type: PartType, amount: u32) -> u32 {
        if let Ok(idx) = self.entries.binary_search_by_key(&id, DeckEntry::id) {
            let ret = self.entries[idx].count_mut(part_type).decrement(amount);
            if self.entries[idx].empty() {
                self.entries.remove(idx);
            }
            ret
        } else {
            0
        }
    }

    pub fn entries(&self) -> impl Iterator<Item = &DeckEntry> {
        self.entries.iter()
    }

    pub fn iter_part(
        &self,
        cards: &'static CardData,
        part: DeckPart,
    ) -> impl Iterator<Item = (Id, usize)> + '_ {
        self.entries()
            .map(move |entry| (entry.id(), entry.count(part.into())))
            .filter(move |(id, count)| *count > 0 && part.can_contain(&cards[*id]))
    }
}

impl TextEncoding for Deck {
    fn encode(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        let mut entries = self.entries.iter();
        if let Some(entry) = entries.next() {
            entry.encode(writer)?;
        }
        for entry in entries {
            writer.write_char(',')?;
            entry.encode(writer)?;
        }

        writer.write_char(' ')?;
        self.undo_redo.encode(writer)
    }

    fn decode(text: &str) -> Option<Self> {
        let (entries, undo_redo) = text.split_once(' ')?;

        let entries = if entries.is_empty() {
            Vec::new()
        } else {
            entries
                .split(',')
                .map(DeckEntry::decode)
                .collect::<Option<_>>()?
        };
        let undo_redo = TextEncoding::decode(undo_redo)?;

        Some(Self { entries, undo_redo })
    }
}

#[cfg(test)]
mod test {
    use std::mem::{align_of, size_of};

    use common::card::{Card, CardDescription, MonsterType};

    use super::*;

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
        let mut deck = Deck::default();

        assert_part_eq!(&deck, PartType::Playing, &[]);
        assert_part_eq!(&deck, PartType::Side, &[]);

        deck.undo();
        assert_part_eq!(&deck, PartType::Playing, &[]);
        assert_part_eq!(&deck, PartType::Side, &[]);

        deck.redo();
        assert_part_eq!(&deck, PartType::Playing, &[]);
        assert_part_eq!(&deck, PartType::Side, &[]);
    }

    #[test]
    fn add_remove() {
        const ID: Id = Id::new(1234);
        const AMOUNT: u32 = 4321;

        for TestCase { current, other } in TestCase::iter() {
            let mut deck = Deck::default();

            deck.increment(ID, current, AMOUNT);
            assert_part_eq!(&deck, current, &[(ID, AMOUNT as usize)]);
            assert_part_eq!(&deck, other, &[]);

            deck.decrement(ID, current, AMOUNT);
            assert_part_eq!(&deck, current, &[]);
            assert_part_eq!(&deck, other, &[]);

            deck.undo();
            assert_part_eq!(&deck, current, &[(ID, AMOUNT as usize)]);
            assert_part_eq!(&deck, other, &[]);

            deck.undo();
            assert_part_eq!(&deck, current, &[]);
            assert_part_eq!(&deck, other, &[]);

            deck.redo();
            assert_part_eq!(&deck, current, &[(ID, AMOUNT as usize)]);
            assert_part_eq!(&deck, other, &[]);

            deck.redo();
            assert_part_eq!(&deck, current, &[]);
            assert_part_eq!(&deck, other, &[]);
        }
    }

    #[test]
    fn remove_on_empty() {
        const ID: Id = Id::new(1234);
        const AMOUNT: u32 = 4321;

        let mut deck = Deck::default();
        deck.decrement(ID, PartType::Playing, AMOUNT);
        deck.decrement(ID, PartType::Side, AMOUNT);

        assert_part_eq!(&deck, PartType::Playing, &[]);
        assert_part_eq!(&deck, PartType::Side, &[]);

        deck.undo();
        assert_part_eq!(&deck, PartType::Playing, &[]);
        assert_part_eq!(&deck, PartType::Side, &[]);

        deck.redo();
        assert_part_eq!(&deck, PartType::Playing, &[]);
        assert_part_eq!(&deck, PartType::Side, &[]);
    }

    #[test]
    fn remove_too_many() {
        const ID: Id = Id::new(1234);
        const AMOUNT: u32 = 4321;
        const REMOVE_AMOUNT: u32 = 9876;

        for TestCase { current, other } in TestCase::iter() {
            let mut deck = Deck::default();
            deck.increment(ID, current, AMOUNT);
            deck.decrement(ID, current, REMOVE_AMOUNT);

            assert_part_eq!(&deck, current, &[]);
            assert_part_eq!(&deck, other, &[]);

            deck.undo();
            assert_part_eq!(&deck, current, &[(ID, AMOUNT as usize)]);
            assert_part_eq!(&deck, other, &[]);

            deck.undo();
            assert_part_eq!(&deck, current, &[]);
            assert_part_eq!(&deck, other, &[]);

            deck.redo();
            assert_part_eq!(&deck, current, &[(ID, AMOUNT as usize)]);
            assert_part_eq!(&deck, other, &[]);

            deck.redo();
            assert_part_eq!(&deck, current, &[]);
            assert_part_eq!(&deck, other, &[]);
        }
    }

    #[test]
    fn add_too_many() {
        const ID: Id = Id::new(1234);
        const AMOUNT: u32 = 4321;

        for TestCase { current, other } in TestCase::iter() {
            let mut deck = Deck::default();
            deck.increment(ID, current, u32::MAX - 1);
            deck.increment(ID, current, AMOUNT);

            assert_part_eq!(&deck, current, &[(ID, u32::MAX as usize)]);
            assert_part_eq!(&deck, other, &[]);

            deck.undo();
            assert_part_eq!(&deck, current, &[(ID, (u32::MAX - 1) as usize)]);
            assert_part_eq!(&deck, other, &[]);

            deck.undo();
            assert_part_eq!(&deck, current, &[]);
            assert_part_eq!(&deck, other, &[]);

            deck.redo();
            assert_part_eq!(&deck, current, &[(ID, (u32::MAX - 1) as usize)]);
            assert_part_eq!(&deck, other, &[]);

            deck.redo();
            assert_part_eq!(&deck, current, &[(ID, u32::MAX as usize)]);
            assert_part_eq!(&deck, other, &[]);
        }
    }

    #[test]
    fn iter_part() {
        const MAIN_ID: Id = Id::new(1234);
        const EXTRA_ID: Id = Id::new(2345);

        let data = {
            let cards = vec![
                Card {
                    name: String::new(),
                    ids: vec![MAIN_ID],
                    description: CardDescription::Regular(Vec::new()),
                    search_text: String::new(),
                    card_type: common::card::CardType::Spell(common::card::SpellType::Normal),
                    limit: common::card::CardLimit::Unlimited,
                    archetype: None,
                },
                Card {
                    name: String::new(),
                    ids: vec![EXTRA_ID],
                    description: CardDescription::Regular(Vec::new()),
                    search_text: String::new(),
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
            ];

            Box::leak(Box::new(CardData::new(cards)))
        };

        let mut deck = Deck::default();
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
        side_cards.sort_by_key(|(id, _)| *id);
        assert_eq!(side_cards, &[(MAIN_ID, 3), (EXTRA_ID, 5)]);
    }

    #[test]
    fn encoding_empty() {
        let deck = Deck::default();
        assert!(Deck::decode(&deck.encode_string())
            .unwrap()
            .entries
            .is_empty());
    }

    #[test]
    fn encoding() {
        const ID: Id = Id::new(1234);
        const OTHER_ID: Id = Id::new(3456);
        const AMOUNT: u32 = 4321;
        const OTHER_AMOUNT: u32 = 6543;

        for TestCase { current, other } in TestCase::iter() {
            let mut deck = Deck::default();
            deck.increment(ID, current, AMOUNT);
            deck.increment(OTHER_ID, current, OTHER_AMOUNT);
            deck.undo();
            deck.undo();
            deck.redo();

            let mut deck = Deck::decode(&deck.encode_string()).unwrap();
            assert_part_eq!(&deck, current, &[(ID, AMOUNT as usize)]);
            assert_part_eq!(&deck, other, []);

            deck.undo();
            assert_part_eq!(&deck, current, []);
            assert_part_eq!(&deck, other, []);

            deck.redo();
            deck.redo();
            assert_part_eq!(
                &deck,
                current,
                &[(ID, AMOUNT as usize), (OTHER_ID, OTHER_AMOUNT as usize)]
            );
            assert_part_eq!(&deck, other, []);
        }
    }
}
