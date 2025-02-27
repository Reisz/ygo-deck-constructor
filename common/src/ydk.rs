use std::io::{self, Write};

use thiserror::Error;

use crate::{
    card::CardPassword,
    card_data::CardData,
    deck::Deck,
    deck_part::{DeckPart, EntriesForPart},
};

/// Section name in the YDK format.
#[must_use]
pub fn ydk_name(part: DeckPart) -> &'static str {
    match part {
        DeckPart::Main => "main",
        DeckPart::Extra => "extra",
        DeckPart::Side => "side",
    }
}

/// Prefix for section header in the YDK format (`#` or `!`)
#[must_use]
pub fn ydk_prefix(part: DeckPart) -> char {
    match part {
        DeckPart::Main | DeckPart::Extra => '#',
        DeckPart::Side => '!',
    }
}

/// Possible errors when reading YDK data.
#[derive(Debug, Error)]
pub enum Error {
    #[error("could not read input")]
    Reader(#[from] io::Error),
    #[error("could not parse input")]
    Parser(#[from] parse::Error),
    #[error("unknown id: {0:?}")]
    UnknownPassword(CardPassword),
}

/// Deserialize a deck from the YDK format used by `YGOPRODeck`.
///
/// Due to the absence of a centralized standard, this implementation is quite lenient.
///
/// # Errors
///
/// If the input can not be parsed, an error is returned.
pub fn load(data: &str, cards: &CardData) -> Result<Deck, Error> {
    let result = parse::parse(data)?;

    let mut deck = Deck::default();
    for part in DeckPart::iter() {
        for &password in &result[part as usize] {
            let id = cards
                .id_for_password(password)
                .ok_or(Error::UnknownPassword(password))?;

            deck.increment(id, part.into(), 1);
        }
    }

    Ok(deck)
}

/// Serialize the deck into the YDK format used by `YGOPRODeck`.
///
/// # Errors
///
/// See [`writeln!`].
pub fn save(deck: &Deck, cards: &CardData, writer: &mut impl Write) -> io::Result<()> {
    for part in DeckPart::iter() {
        writeln!(writer, "{}{}", ydk_prefix(part), ydk_name(part))?;

        for (id, count) in deck.entries().for_part(part, cards) {
            for _ in 0..count {
                writeln!(writer, "{}", cards[id].password)?;
            }
        }
    }

    Ok(())
}

mod parse {
    use nom::{
        Finish, Parser,
        branch::alt,
        bytes::complete::tag,
        character::complete::{self as character, multispace0, multispace1, one_of},
        combinator::opt,
        multi::separated_list1,
        sequence::{delimited, pair, preceded},
    };

    use crate::{card::CardPassword, deck_part::DeckPart};

    use super::ydk_name;

    pub type Error = nom::error::Error<String>;
    pub type Result<T> = std::result::Result<T, nom::error::Error<String>>;
    type IResult<'a, T> = nom::IResult<&'a str, T>;

    trait IParser<'a, T>: Parser<&'a str, Output = T, Error = nom::error::Error<&'a str>> {}
    impl<'a, T, U: Parser<&'a str, Output = T, Error = nom::error::Error<&'a str>>> IParser<'a, T>
        for U
    {
    }

    fn id(input: &str) -> IResult<CardPassword> {
        character::u32.parse(input)
    }

    fn ids(input: &str) -> IResult<Vec<CardPassword>> {
        separated_list1(multispace1, id).parse(input)
    }

    fn header_impl<'a>(part: DeckPart) -> impl IParser<'a, DeckPart> {
        pair(one_of("#!"), tag(ydk_name(part))).map(move |_| part)
    }

    fn header(input: &str) -> IResult<DeckPart> {
        alt((
            header_impl(DeckPart::Main),
            header_impl(DeckPart::Extra),
            header_impl(DeckPart::Side),
        ))
        .parse(input)
    }

    fn section(input: &str) -> IResult<(DeckPart, Vec<CardPassword>)> {
        pair(header, opt(preceded(multispace1, ids)))
            .map(|(part, ids)| (part, ids.unwrap_or_default()))
            .parse(input)
    }

    fn deck(input: &str) -> IResult<[Vec<CardPassword>; 3]> {
        separated_list1(multispace1, section)
            .map(|parts| {
                let mut deck = [vec![], vec![], vec![]];
                for (part_type, content) in parts {
                    deck[part_type as usize].extend(&content);
                }
                deck
            })
            .parse(input)
    }

    pub fn parse(input: &str) -> Result<[Vec<CardPassword>; 3]> {
        delimited(multispace0, deck, multispace0)
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

    use crate::{
        card::test_util::{make_card, make_extra_deck_card},
        card_data::{CardDataStorage, Id},
    };

    use super::*;

    struct YdkData {
        deck: Deck,
        ydk: String,
    }

    const PASSWDS: [[(u16, u32); 4]; 3] = [
        [(0, 1), (1, 23), (2, 456), (3, 7890)],
        [(8, 2), (9, 24), (10, 457), (11, 7891)],
        [(4, 3), (5, 25), (6, 458), (7, 7892)],
    ];

    impl YdkData {
        fn get() -> Vec<Self> {
            const MAX: usize = 3;

            let mut result = Vec::new();

            let mut numbers = [
                PASSWDS[0].iter().copied().cycle(),
                PASSWDS[1].iter().copied().cycle(),
                PASSWDS[2].iter().copied().cycle(),
            ];

            for (main_count, extra_count, side_count) in iproduct!(0..=MAX, 0..=MAX, 0..=MAX) {
                let mut deck = Deck::default();
                let mut ydk = Vec::new();

                for (part, count) in [
                    (DeckPart::Main, main_count),
                    (DeckPart::Extra, extra_count),
                    (DeckPart::Side, side_count),
                ] {
                    let mut ids = Vec::new();

                    match part {
                        DeckPart::Main => ydk.push("#main".to_owned()),
                        DeckPart::Extra => ydk.push("#extra".to_owned()),
                        DeckPart::Side => ydk.push("!side".to_owned()),
                    }

                    for _ in 0..count {
                        let number = numbers[part as usize].next().unwrap();
                        deck.increment(Id::new(number.0), part.into(), 1);
                        ids.push(number.1);
                    }

                    ids.sort_unstable();
                    ydk.extend(ids.into_iter().map(|id| format!("{id}")));
                }

                let mut ydk = ydk.join("\n");
                ydk.push('\n');

                result.push(YdkData { deck, ydk });
            }

            result
        }
    }

    fn card_data() -> CardData {
        let mut data = Vec::new();

        for password in PASSWDS[0].iter().chain(PASSWDS[2].iter()) {
            data.push(make_card(password.1));
        }

        for password in &PASSWDS[1] {
            data.push(make_extra_deck_card(password.1));
        }

        CardDataStorage::new(data, vec![]).into()
    }

    #[test]
    fn ydk_serialization() {
        for data in YdkData::get() {
            let mut output = Vec::new();
            save(&data.deck, &card_data(), &mut output).unwrap();
            assert_eq!(data.ydk, String::from_utf8(output).unwrap());
        }
    }

    #[test]
    fn ydk_deserialization() {
        for data in YdkData::get() {
            let deck = load(&data.ydk, &card_data()).unwrap();
            itertools::assert_equal(data.deck.entries(), deck.entries());
        }
    }
}
