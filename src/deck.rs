use std::{fmt, ops::Deref};

use common::{
    card_data::{CardData, Id},
    deck::{DeckEntry, PartType},
};
use leptos::expect_context;

use crate::{
    text_encoding::TextEncoding,
    undo_redo::{UndoRedo, UndoRedoMessage},
};

impl TextEncoding for DeckEntry {
    fn encode(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        let cards = expect_context::<&'static CardData>();

        let password = cards[self.id()].passwords[0];
        let playing = self.count(PartType::Playing);
        let side = self.count(PartType::Side);
        write!(writer, "{password}:{playing}:{side}")
    }

    fn decode(text: &str) -> Option<Self> {
        let cards = expect_context::<&'static CardData>();

        let (password, text) = text.split_once(':')?;
        let (playing, side) = text.split_once(':')?;

        let id = cards.id_for_password(password.parse().ok()?)?;
        let playing = playing.parse().ok()?;
        let side = side.parse().ok()?;

        let mut result = Self::new(id);
        result.set_count(PartType::Playing, playing);
        result.set_count(PartType::Side, side);
        Some(result)
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
    deck: common::deck::Deck,
    undo_redo: UndoRedo<DeckMessage>,
}

impl Deck {
    #[must_use]
    pub fn new(deck: common::deck::Deck) -> Self {
        Self {
            deck,
            undo_redo: UndoRedo::default(),
        }
    }

    pub fn increment(&mut self, id: Id, part_type: PartType, amount: u8) {
        let amount = self.deck.increment(id, part_type, amount);
        if amount > 0 {
            self.undo_redo
                .push_action(DeckMessage::Inc(id, part_type, amount));
        }
    }

    pub fn decrement(&mut self, id: Id, part_type: PartType, amount: u8) {
        let amount = self.deck.decrement(id, part_type, amount);
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
                debug_assert_eq!(amount, self.deck.increment(id, part_type, amount));
            }
            DeckMessage::Dec(id, part_type, amount) => {
                debug_assert_eq!(amount, self.deck.decrement(id, part_type, amount));
            }
        }
    }
}

impl Deref for Deck {
    type Target = common::deck::Deck;

    fn deref(&self) -> &Self::Target {
        &self.deck
    }
}

impl TextEncoding for Deck {
    fn encode(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        let mut entries = self.deck.entries();
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
        let deck = common::deck::Deck::new(entries);

        let undo_redo = TextEncoding::decode(undo_redo)?;

        Some(Self { deck, undo_redo })
    }
}

#[cfg(test)]
mod test {
    use common::{assert_part_eq, card::test_util::make_card};
    use leptos::provide_context;

    use super::*;

    #[test]
    fn encoding_empty() {
        let deck = Deck::default();
        assert_eq!(
            Deck::decode(&deck.encode_string())
                .unwrap()
                .entries()
                .count(),
            0
        );
    }

    #[test]
    fn encoding() {
        const ID: Id = Id::new(0);
        const OTHER_ID: Id = Id::new(1);
        const AMOUNT: u8 = 43;
        const OTHER_AMOUNT: u8 = 65;

        let card_data = CardData::new(vec![make_card(1234), make_card(9876)]);
        let card_data: &'static CardData = Box::leak(Box::new(card_data));
        provide_context(card_data);

        for (current, other) in [
            (PartType::Playing, PartType::Side),
            (PartType::Side, PartType::Playing),
        ] {
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
