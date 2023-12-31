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

#[cfg(test)]
mod test {
    use std::assert_matches::assert_matches;

    use crate::undo_redo::UndoRedo;

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
}
