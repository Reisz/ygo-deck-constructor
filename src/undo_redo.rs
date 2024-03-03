use std::fmt;

use crate::text_encoding::TextEncoding;

pub trait UndoRedoMessage: Copy {
    #[must_use]
    fn invert(self) -> Self;
}

#[derive(Debug, Clone)]
pub struct UndoRedo<T> {
    entries: Vec<T>,
    offset: usize,
}

impl<T> Default for UndoRedo<T> {
    fn default() -> Self {
        Self {
            entries: Vec::default(),
            offset: 0,
        }
    }
}

impl<T: UndoRedoMessage> UndoRedo<T> {
    pub fn push_action(&mut self, action: T) {
        if self.offset > 0 {
            self.entries.truncate(self.entries.len() - self.offset);
            self.offset = 0;
        }

        self.entries.push(action);
    }

    #[must_use]
    pub fn undo(&mut self) -> Option<T> {
        let message = self.entries.iter().copied().rev().nth(self.offset);
        if message.is_some() {
            self.offset += 1;
        }
        message.map(UndoRedoMessage::invert)
    }

    #[must_use]
    pub fn redo(&mut self) -> Option<T> {
        if self.offset > 0 {
            self.offset -= 1;
            self.entries.iter().copied().rev().nth(self.offset)
        } else {
            None
        }
    }
}

impl<T: TextEncoding> TextEncoding for UndoRedo<T> {
    fn encode(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        write!(writer, "{};", self.offset)?;

        let mut entries = self.entries.iter();
        if let Some(item) = entries.next() {
            item.encode(writer)?;
        }
        for item in entries {
            writer.write_char(',')?;
            item.encode(writer)?;
        }

        Ok(())
    }

    fn decode(text: &str) -> Option<Self> {
        let (offset, text) = text.split_once(';')?;

        let offset = offset.parse().ok()?;
        let entries = if text.is_empty() {
            Vec::new()
        } else {
            text.split(',').map(T::decode).collect::<Option<_>>()?
        };

        Some(Self { entries, offset })
    }
}

#[cfg(test)]
mod test {
    use std::assert_matches::assert_matches;

    use super::*;

    #[derive(Debug, Clone, Copy)]
    enum TestMessage {
        Apply(usize),
        Revert(usize),
    }

    impl UndoRedoMessage for TestMessage {
        fn invert(self) -> Self {
            match self {
                Self::Apply(idx) => Self::Revert(idx),
                Self::Revert(idx) => Self::Apply(idx),
            }
        }
    }

    impl TextEncoding for TestMessage {
        fn encode(&self, writer: &mut impl fmt::Write) -> fmt::Result {
            let sign = match self {
                Self::Apply(_) => 'a',
                Self::Revert(_) => 'r',
            };

            let (Self::Apply(value) | Self::Revert(value)) = self;
            write!(writer, "{sign}{value}")
        }

        fn decode(text: &str) -> Option<Self> {
            assert!(text.starts_with(['a', 'r']));
            let (sign, text) = text.split_at(1);
            let value = text.parse().unwrap();

            let result = match sign {
                "a" => Self::Apply(value),
                "r" => Self::Revert(value),
                _ => unreachable!(),
            };

            Some(result)
        }
    }

    type UR = UndoRedo<TestMessage>;

    #[test]
    fn empty() {
        let mut ur = UR::default();
        assert_matches!(ur.undo(), None);
        assert_matches!(ur.redo(), None);
    }

    #[test]
    fn one_message() {
        let mut ur = UR::default();
        ur.push_action(TestMessage::Apply(0));

        assert_matches!(ur.undo(), Some(TestMessage::Revert(0)));
        assert_matches!(ur.redo(), Some(TestMessage::Apply(0)));
    }

    #[test]
    fn multiple_messages() {
        let mut ur = UR::default();
        ur.push_action(TestMessage::Apply(0));
        ur.push_action(TestMessage::Apply(1));

        assert_matches!(ur.undo(), Some(TestMessage::Revert(1)));
        assert_matches!(ur.undo(), Some(TestMessage::Revert(0)));
        assert_matches!(ur.redo(), Some(TestMessage::Apply(0)));
        assert_matches!(ur.redo(), Some(TestMessage::Apply(1)));
    }

    #[test]
    fn extra_undo_is_noop() {
        let mut ur = UR::default();
        ur.push_action(TestMessage::Apply(0));

        assert_matches!(ur.undo(), Some(TestMessage::Revert(0)));
        assert_matches!(ur.undo(), None);
        assert_matches!(ur.redo(), Some(TestMessage::Apply(0)));
    }

    #[test]
    fn extra_redo_is_noop() {
        let mut ur = UR::default();
        ur.push_action(TestMessage::Apply(0));

        assert_matches!(ur.redo(), None);
        assert_matches!(ur.undo(), Some(TestMessage::Revert(0)));
    }

    #[test]
    fn push_clears_redo() {
        let mut ur = UR::default();
        ur.push_action(TestMessage::Apply(0));
        ur.push_action(TestMessage::Apply(1));

        assert_matches!(ur.undo(), Some(TestMessage::Revert(1)));
        ur.push_action(TestMessage::Apply(2));

        assert_matches!(ur.redo(), None);
        assert_matches!(ur.undo(), Some(TestMessage::Revert(2)));
        assert_matches!(ur.undo(), Some(TestMessage::Revert(0)));
        assert_matches!(ur.undo(), None);
    }

    #[test]
    fn encoding() {
        let mut ur = UR::default();
        ur.push_action(TestMessage::Apply(0));
        ur.push_action(TestMessage::Apply(1));

        let mut ur = UR::decode(&ur.encode_string()).unwrap();
        assert_matches!(ur.undo(), Some(TestMessage::Revert(1)));

        let mut ur = UR::decode(&ur.encode_string()).unwrap();
        assert_matches!(ur.undo(), Some(TestMessage::Revert(0)));

        let mut ur = UR::decode(&ur.encode_string()).unwrap();
        assert_matches!(ur.redo(), Some(TestMessage::Apply(0)));

        let mut ur = UR::decode(&ur.encode_string()).unwrap();
        assert_matches!(ur.redo(), Some(TestMessage::Apply(1)));
    }
}
