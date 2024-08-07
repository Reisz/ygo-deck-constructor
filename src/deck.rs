use std::fmt;

use common::card_data::{CardData, Id};
use leptos::expect_context;

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

impl TextEncoding for DeckEntry {
    fn encode(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        let cards = expect_context::<&'static CardData>();

        let password = cards[self.id].passwords[0];
        let [playing, side] = self.counts;
        write!(writer, "{password}:{playing}:{side}")
    }

    fn decode(text: &str) -> Option<Self> {
        let cards = expect_context::<&'static CardData>();

        let (password, text) = text.split_once(':')?;
        let (playing, side) = text.split_once(':')?;

        let id = cards.id_for_password(password.parse().ok()?)?;
        let playing = playing.parse().ok()?;
        let side = side.parse().ok()?;

        Some(Self {
            id,
            counts: [playing, side],
        })
    }
}

impl DeckEntry {
    fn new(id: Id) -> Self {
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
}

#[derive(Debug, Clone, Copy)]
enum DeckMessage {
    Inc(Id, PartType, u8),
    Dec(Id, PartType, u8),
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
        let cards = expect_context::<&'static CardData>();

        let sign = match self {
            Self::Inc(..) => '+',
            Self::Dec(..) => '-',
        };

        let (Self::Inc(id, part, count) | Self::Dec(id, part, count)) = self;
        let part = match part {
            PartType::Playing => 'p',
            PartType::Side => 's',
        };

        write!(writer, "{sign}{part}{}:{count}", cards[*id].passwords[0])
    }

    fn decode(text: &str) -> Option<Self> {
        let cards = expect_context::<&'static CardData>();

        text.starts_with(['+', '-']).then_some(())?;
        let (sign, text) = text.split_at(1);

        text.starts_with(['p', 's']).then_some(())?;
        let (part, text) = text.split_at(1);

        let (password, count) = text.split_once(':')?;

        let id = cards.id_for_password(password.parse().ok()?)?;
        let part = match part {
            "p" => PartType::Playing,
            "s" => PartType::Side,
            _ => return None,
        };
        let count = count.parse().ok()?;

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
    pub fn increment(&mut self, id: Id, part_type: PartType, amount: u8) {
        let amount = self.increment_internal(id, part_type, amount);
        if amount > 0 {
            self.undo_redo
                .push_action(DeckMessage::Inc(id, part_type, amount));
        }
    }

    pub fn decrement(&mut self, id: Id, part_type: PartType, amount: u8) {
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

    fn increment_internal(&mut self, id: Id, part_type: PartType, amount: u8) -> u8 {
        let idx = self
            .entries
            .binary_search_by_key(&id, DeckEntry::id)
            .unwrap_or_else(|idx| {
                self.entries.insert(idx, DeckEntry::new(id));
                idx
            });

        let entry = &mut self.entries[idx].counts[part_type.idx()];

        if let Some(new_val) = entry.checked_add(amount) {
            *entry = new_val;
            return amount;
        }

        let ret = u8::MAX - *entry;
        *entry = u8::MAX;
        ret
    }

    fn decrement_internal(&mut self, id: Id, part_type: PartType, amount: u8) -> u8 {
        if let Ok(idx) = self.entries.binary_search_by_key(&id, DeckEntry::id) {
            let entry = &mut self.entries[idx].counts[part_type.idx()];

            let ret = if let Some(new_val) = entry.checked_sub(amount) {
                *entry = new_val;
                amount
            } else {
                let ret = *entry;
                *entry = 0;
                ret
            };

            if self.entries[idx].counts.iter().all(|count| *count == 0) {
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

    use common::card::{
        Card, CardDescription, CardLimit, CardPassword, CardType, MonsterType, TrapType,
    };
    use leptos::provide_context;

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
        assert!(size_of::<DeckEntry>() <= 4);
        assert!(align_of::<DeckEntry>() <= 4);
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
        const AMOUNT: u8 = 43;

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
        const AMOUNT: u8 = 43;

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
        const AMOUNT: u8 = 43;
        const REMOVE_AMOUNT: u8 = 98;

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
        const AMOUNT: u8 = 43;

        for TestCase { current, other } in TestCase::iter() {
            let mut deck = Deck::default();
            deck.increment(ID, current, u8::MAX - 1);
            deck.increment(ID, current, AMOUNT);

            assert_part_eq!(&deck, current, &[(ID, u8::MAX as usize)]);
            assert_part_eq!(&deck, other, &[]);

            deck.undo();
            assert_part_eq!(&deck, current, &[(ID, (u8::MAX - 1) as usize)]);
            assert_part_eq!(&deck, other, &[]);

            deck.undo();
            assert_part_eq!(&deck, current, &[]);
            assert_part_eq!(&deck, other, &[]);

            deck.redo();
            assert_part_eq!(&deck, current, &[(ID, (u8::MAX - 1) as usize)]);
            assert_part_eq!(&deck, other, &[]);

            deck.redo();
            assert_part_eq!(&deck, current, &[(ID, u8::MAX as usize)]);
            assert_part_eq!(&deck, other, &[]);
        }
    }

    #[test]
    fn iter_part() {
        const MAIN_PASSWD: CardPassword = 1234;
        const EXTRA_PASSWD: CardPassword = 2345;

        const MAIN_ID: Id = Id::new(0);
        const EXTRA_ID: Id = Id::new(1);

        let data = {
            let cards = vec![
                Card {
                    name: String::new(),
                    passwords: vec![MAIN_PASSWD],
                    description: CardDescription::Regular(Vec::new()),
                    search_text: String::new(),
                    card_type: common::card::CardType::Spell(common::card::SpellType::Normal),
                    limit: common::card::CardLimit::Unlimited,
                    archetype: None,
                },
                Card {
                    name: String::new(),
                    passwords: vec![EXTRA_PASSWD],
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
        const ID: Id = Id::new(0);
        const OTHER_ID: Id = Id::new(1);
        const AMOUNT: u8 = 43;
        const OTHER_AMOUNT: u8 = 65;

        let card_data = CardData::new(vec![
            Card {
                name: String::new(),
                passwords: vec![1234],
                description: CardDescription::Regular(Vec::new()),
                search_text: String::new(),
                card_type: CardType::Trap(TrapType::Normal),
                limit: CardLimit::Unlimited,
                archetype: None,
            },
            Card {
                name: String::new(),
                passwords: vec![9876],
                description: CardDescription::Regular(Vec::new()),
                search_text: String::new(),
                card_type: CardType::Trap(TrapType::Normal),
                limit: CardLimit::Unlimited,
                archetype: None,
            },
        ]);
        let card_data: &'static CardData = Box::leak(Box::new(card_data));
        provide_context(card_data);

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
