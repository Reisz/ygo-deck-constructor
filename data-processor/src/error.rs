use std::{error::Error, fmt};

use common::card::CardPassword;

use crate::ygoprodeck;

#[derive(Debug, Clone)]
pub struct ProcessingError {
    password: CardPassword,
    field: &'static str,
    error: ProjectionErrorKind,
}

impl ProcessingError {
    pub fn new_unexpected(
        password: CardPassword,
        field: &'static str,
        value: &(impl fmt::Debug + ?Sized),
    ) -> Self {
        Self {
            password,
            field,
            error: ProjectionErrorKind::UnexpectedValue(format!("{value:?}")),
        }
    }

    pub fn new_unknown(
        password: CardPassword,
        field: &'static str,
        value: &(impl fmt::Debug + ?Sized),
    ) -> Self {
        Self {
            password,
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
            "Card password {password} <{}?id={password}>",
            ygoprodeck::URL,
            password = self.password
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
    fn try_unwrap_field(
        self,
        password: CardPassword,
        field: &'static str,
    ) -> Result<T, ProcessingError>;
}

impl<T> TryUnwrapField<T> for Option<T> {
    fn try_unwrap_field(
        self,
        password: CardPassword,
        field: &'static str,
    ) -> Result<T, ProcessingError> {
        self.ok_or(ProcessingError {
            password,
            field,
            error: ProjectionErrorKind::MissingField,
        })
    }
}
