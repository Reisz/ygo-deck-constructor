// TODO remove once actually used
#![allow(unused)]

use std::{
    io::{self, Read, Write},
    ops::{Index, IndexMut},
};

use common::card::Id;
use thiserror::Error;

use crate::deck::PartType;

/// Section name in the YDK format.
#[must_use]
pub fn ydk_name(part: PartType) -> &'static str {
    match part {
        PartType::Main => "main",
        PartType::Extra => "extra",
        PartType::Side => "side",
    }
}

/// Prefix for section header in the YDK format (`#` or `!`)
#[must_use]
pub fn ydk_prefix(part: PartType) -> char {
    match part {
        PartType::Main | PartType::Extra => '#',
        PartType::Side => '!',
    }
}

/// Possible errors when reading YDK data.
#[derive(Debug, Error)]
pub enum Error {
    #[error("could not read input")]
    Reader(#[from] io::Error),
    #[error("could not parse input")]
    Parser(#[from] parse::Error),
}

/// A single Yu-Gi-Oh deck.
///
/// Content is accessed by indexing with the [`Part`] type.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Deck {
    parts: [Vec<Id>; 3],
}

impl Deck {
    /// Deserialize a deck from the YDK format used by `YGOPRODeck`.
    ///
    /// Due to the absence of a centralized standard, this implementation is quite lenient.
    ///
    /// # Errors
    ///
    /// If the input can not be parsed, an error is returned.
    ///
    /// See [`Read::read_to_string`] for other error semantics.
    pub fn from_ydk(reader: &mut impl Read) -> Result<Self, Error> {
        let mut data = String::new();
        reader.read_to_string(&mut data)?;
        Ok(Self {
            parts: parse::parse(&data)?,
        })
    }

    /// Serialize the deck into the YDK format used by `YGOPRODeck`.
    ///
    /// # Errors
    ///
    /// See [`writeln!`].
    pub fn to_ydk(&self, writer: &mut impl Write) -> io::Result<()> {
        for part in [PartType::Main, PartType::Extra, PartType::Side] {
            writeln!(writer, "{}{}", ydk_prefix(part), ydk_name(part))?;

            for card in &self[part] {
                writeln!(writer, "{}", card.get())?;
            }
        }

        Ok(())
    }
}

impl Index<PartType> for Deck {
    type Output = Vec<Id>;

    fn index(&self, index: PartType) -> &Self::Output {
        &self.parts[index as usize]
    }
}

impl IndexMut<PartType> for Deck {
    fn index_mut(&mut self, index: PartType) -> &mut Self::Output {
        &mut self.parts[index as usize]
    }
}

mod parse {
    use common::card::Id;
    use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::{self as character, line_ending, one_of},
        combinator::opt,
        multi::{many0, many1, separated_list0, separated_list1},
        sequence::{pair, preceded, terminated},
        Finish, Parser,
    };

    use crate::deck::PartType;

    use super::ydk_name;

    pub type Error = nom::error::Error<String>;
    pub type Result<T> = std::result::Result<T, nom::error::Error<String>>;
    type IResult<'a, T> = nom::IResult<&'a str, T>;

    trait IParser<'a, T>: Parser<&'a str, T, nom::error::Error<&'a str>> {}
    impl<'a, T, U: Parser<&'a str, T, nom::error::Error<&'a str>>> IParser<'a, T> for U {}

    fn sep(input: &str) -> IResult<()> {
        many1(line_ending).map(|_| ()).parse(input)
    }

    fn id(input: &str) -> IResult<Id> {
        character::u64.map(Id::new).parse(input)
    }

    fn ids(input: &str) -> IResult<Vec<Id>> {
        separated_list1(sep, id)(input)
    }

    fn header_impl<'a>(part: PartType) -> impl IParser<'a, PartType> {
        pair(one_of("#!"), tag(ydk_name(part))).map(move |_| part)
    }

    fn header(input: &str) -> IResult<PartType> {
        alt((
            header_impl(PartType::Main),
            header_impl(PartType::Extra),
            header_impl(PartType::Side),
        ))(input)
    }

    fn section(input: &str) -> IResult<(PartType, Vec<Id>)> {
        pair(header, opt(preceded(sep, ids)))
            .map(|(part, ids)| (part, ids.unwrap_or_default()))
            .parse(input)
    }

    fn deck(input: &str) -> IResult<[Vec<Id>; 3]> {
        separated_list0(sep, section)
            .map(|parts| {
                let mut deck = [vec![], vec![], vec![]];
                for (part_type, content) in parts {
                    deck[part_type as usize].extend(&content);
                }
                deck
            })
            .parse(input)
    }

    pub fn parse(input: &str) -> Result<[Vec<Id>; 3]> {
        terminated(deck, many0(line_ending))
            .parse(input)
            .finish()
            .map_err(|nom::error::Error { input, code }| Error {
                input: input.to_owned(),
                code,
            })
            .map(|(_, res)| res)
    }
}

#[cfg(test)]
mod test {
    use itertools::iproduct;

    use super::*;

    struct YdkData {
        deck: Deck,
        ydk: Vec<String>,
    }

    impl YdkData {
        fn get() -> Vec<Self> {
            const MAX: usize = 3;

            let mut result = Vec::new();

            let mut numbers = [1, 23, 456, 7890].into_iter().cycle();

            for (main_count, extra_count, side_count) in iproduct!(0..=MAX, 0..=MAX, 0..=MAX) {
                let mut deck = Deck::default();
                let mut ydk = Vec::new();

                for (part, count) in [
                    (PartType::Main, main_count),
                    (PartType::Extra, extra_count),
                    (PartType::Side, side_count),
                ] {
                    match part {
                        PartType::Main => ydk.push("#main".to_owned()),
                        PartType::Extra => ydk.push("#extra".to_owned()),
                        PartType::Side => ydk.push("!side".to_owned()),
                    }

                    for _ in 0..count {
                        let number = numbers.next().unwrap();
                        deck[part].push(Id::new(number));
                        ydk.push(format!("{number}"));
                    }
                }

                result.push(YdkData { deck, ydk });
            }

            result
        }
    }

    #[test]
    fn ydk_serialization() {
        for data in YdkData::get() {
            let mut output = Vec::new();
            data.deck.to_ydk(&mut output).unwrap();
            itertools::assert_equal(
                data.ydk.iter(),
                String::from_utf8(output)
                    .unwrap()
                    .lines()
                    .filter(|l| !l.is_empty()),
            );
        }
    }

    #[test]
    fn ydk_deserialization() {
        for data in YdkData::get() {
            assert_eq!(
                data.deck,
                Deck::from_ydk(&mut data.ydk.join("\n").as_bytes()).unwrap()
            );
        }
    }
}
