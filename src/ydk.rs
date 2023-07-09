// TODO remove once actually used
#![allow(unused)]

use std::{
    io::{self, Read, Write},
    ops::{Index, IndexMut},
};

use common::card::Id;
use thiserror::Error;

/// The three parts of a Yu-Gi-Oh deck.
#[derive(Debug, Clone, Copy)]
pub enum Part {
    Main,
    Extra,
    Side,
}

impl Part {
    /// Iterate over all elements.
    pub fn iter() -> impl Iterator<Item = Self> {
        [Self::Main, Self::Extra, Self::Side].into_iter()
    }

    /// Section name in the YDK format.
    #[must_use]
    pub fn ydk_name(self) -> &'static str {
        match self {
            Self::Main => "main",
            Self::Extra => "extra",
            Self::Side => "side",
        }
    }

    /// Prefix for section header in the YDK format (`#` or `!`)
    #[must_use]
    pub fn ydk_prefix(self) -> char {
        match self {
            Self::Main | Self::Extra => '#',
            Self::Side => '!',
        }
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
        Ok(parse::parse(&data)?)
    }

    /// Serialize the deck into the YDK format used by `YGOPRODeck`.
    ///
    /// # Errors
    ///
    /// See [`writeln!`].
    pub fn to_ydk(&self, writer: &mut impl Write) -> io::Result<()> {
        for part in Part::iter() {
            writeln!(writer, "{}{}", part.ydk_prefix(), part.ydk_name())?;

            for card in &self[part] {
                writeln!(writer, "{}", card.get())?;
            }
        }

        Ok(())
    }
}

impl Index<Part> for Deck {
    type Output = Vec<Id>;

    fn index(&self, index: Part) -> &Self::Output {
        &self.parts[index as usize]
    }
}

impl IndexMut<Part> for Deck {
    fn index_mut(&mut self, index: Part) -> &mut Self::Output {
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

    use super::{Deck, Part};

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

    fn header_impl(name: &str, part: Part) -> impl IParser<Part> {
        pair(one_of("#!"), tag(name)).map(move |_| part)
    }

    fn header(input: &str) -> IResult<Part> {
        alt((
            header_impl("main", Part::Main),
            header_impl("extra", Part::Extra),
            header_impl("side", Part::Side),
        ))(input)
    }

    fn section(input: &str) -> IResult<Deck> {
        pair(header, opt(preceded(sep, ids)))
            .map(|(part, ids)| {
                let mut deck = Deck::default();
                if let Some(ids) = ids {
                    deck[part] = ids;
                }
                deck
            })
            .parse(input)
    }

    fn deck(input: &str) -> IResult<Deck> {
        separated_list0(sep, section)
            .map(|decks| {
                decks.into_iter().fold(Deck::default(), |mut a, b| {
                    for part in Part::iter() {
                        a[part].extend(&b[part]);
                    }
                    a
                })
            })
            .parse(input)
    }

    pub fn parse(input: &str) -> Result<Deck> {
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

                for (part, count) in Part::iter().zip([main_count, extra_count, side_count]) {
                    match part {
                        Part::Main => ydk.push("#main".to_owned()),
                        Part::Extra => ydk.push("#extra".to_owned()),
                        Part::Side => ydk.push("!side".to_owned()),
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
