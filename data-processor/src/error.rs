use std::{error::Error, fmt};

use crate::ygoprodeck;

#[derive(Debug, Clone)]
pub struct ProcessingError {
    card_id: u64,
    field: &'static str,
    error: ProjectionErrorKind,
}

impl ProcessingError {
    pub fn new_unexpected(
        card_id: u64,
        field: &'static str,
        value: &(impl fmt::Debug + ?Sized),
    ) -> Self {
        Self {
            card_id,
            field,
            error: ProjectionErrorKind::UnexpectedValue(format!("{value:?}")),
        }
    }

    pub fn new_unknown(
        card_id: u64,
        field: &'static str,
        value: &(impl fmt::Debug + ?Sized),
    ) -> Self {
        Self {
            card_id,
            field,
            error: ProjectionErrorKind::UnknownValue(format!("{value:?}")),
        }
    }
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Error projecting field \"{}\"", self.field)?;
        writeln!(
            f,
            "Card id {id} <{}?id={id}>",
            ygoprodeck::URL,
            id = self.card_id
        )?;
        write!(f, "{}", self.error)
    }
}

impl Error for ProcessingError {}

#[derive(Debug, Clone)]
pub enum ProjectionErrorKind {
    MissingField,
    UnexpectedValue(String),
    UnknownValue(String),
}

impl fmt::Display for ProjectionErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::MissingField => write!(f, "Field not present"),
            Self::UnexpectedValue(value) => write!(f, "Unexpected value: {value}"),
            Self::UnknownValue(value) => write!(f, "Unknown value: \"{value}\""),
        }
    }
}

pub trait TryUnwrapField<T> {
    fn try_unwrap_field(self, card_id: u64, field: &'static str) -> Result<T, ProcessingError>;
}

impl<T> TryUnwrapField<T> for Option<T> {
    fn try_unwrap_field(self, card_id: u64, field: &'static str) -> Result<T, ProcessingError> {
        self.ok_or(ProcessingError {
            card_id,
            field,
            error: ProjectionErrorKind::MissingField,
        })
    }
}
