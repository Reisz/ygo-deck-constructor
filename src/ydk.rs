use std::io::{self, Write};

use common::{card::Id, card_data::CardData};
use thiserror::Error;

use crate::{deck::Deck, deck_part::DeckPart};

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
    UnknownId(Id),
}

/// Deserialize a deck from the YDK format used by `YGOPRODeck`.
///
/// Due to the absence of a centralized standard, this implementation is quite lenient.
///
/// # Errors
///
/// If the input can not be parsed, an error is returned.
pub fn load(data: &str, cards: &'static CardData) -> Result<Deck, Error> {
    let result = parse::parse(data)?;

    let mut deck = Deck::default();
    for part in DeckPart::iter() {
        for id in &result[part as usize] {
            let id = cards.normalize(*id);

            if !cards.contains(id) {
                return Err(Error::UnknownId(id));
            }

            deck.increment(id, part.into(), 1);
        }
    }

    deck.reset_history();
    Ok(deck)
}

/// Serialize the deck into the YDK format used by `YGOPRODeck`.
///
/// # Errors
///
/// See [`writeln!`].
pub fn save(deck: &Deck, cards: &'static CardData, writer: &mut impl Write) -> io::Result<()> {
    for part in DeckPart::iter() {
        writeln!(writer, "{}{}", ydk_prefix(part), ydk_name(part))?;

        for (id, count) in deck.iter_part(cards, part) {
            for _ in 0..count {
                writeln!(writer, "{id}")?;
            }
        }
    }

    Ok(())
}

mod parse {
    use common::card::Id;
    use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::{self as character, multispace0, multispace1, one_of},
        combinator::opt,
        multi::separated_list1,
        sequence::{delimited, pair, preceded},
        Finish, Parser,
    };

    use crate::deck_part::DeckPart;

    use super::ydk_name;

    pub type Error = nom::error::Error<String>;
    pub type Result<T> = std::result::Result<T, nom::error::Error<String>>;
    type IResult<'a, T> = nom::IResult<&'a str, T>;

    trait IParser<'a, T>: Parser<&'a str, T, nom::error::Error<&'a str>> {}
    impl<'a, T, U: Parser<&'a str, T, nom::error::Error<&'a str>>> IParser<'a, T> for U {}

    fn id(input: &str) -> IResult<Id> {
        character::u64.map(Id::new).parse(input)
    }

    fn ids(input: &str) -> IResult<Vec<Id>> {
        separated_list1(multispace1, id)(input)
    }

    fn header_impl<'a>(part: DeckPart) -> impl IParser<'a, DeckPart> {
        pair(one_of("#!"), tag(ydk_name(part))).map(move |_| part)
    }

    fn header(input: &str) -> IResult<DeckPart> {
        alt((
            header_impl(DeckPart::Main),
            header_impl(DeckPart::Extra),
            header_impl(DeckPart::Side),
        ))(input)
    }

    fn section(input: &str) -> IResult<(DeckPart, Vec<Id>)> {
        pair(header, opt(preceded(multispace1, ids)))
            .map(|(part, ids)| (part, ids.unwrap_or_default()))
            .parse(input)
    }

    fn deck(input: &str) -> IResult<[Vec<Id>; 3]> {
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

    pub fn parse(input: &str) -> Result<[Vec<Id>; 3]> {
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
    use common::card::{
        Attribute::Dark, Card, CardDescription, CardLimit, CardType, LinkMarkers, MonsterEffect,
        Race::Machine, SpellType,
    };
    use itertools::iproduct;

    use super::*;

    struct YdkData {
        deck: Deck,
        ydk: String,
    }

    const IDS: [[u64; 4]; 3] = [[1, 23, 456, 7890], [2, 24, 457, 7891], [3, 25, 458, 7892]];

    impl YdkData {
        fn get() -> Vec<Self> {
            const MAX: usize = 3;

            let mut result = Vec::new();

            let mut numbers = [
                IDS[0].iter().copied().cycle(),
                IDS[1].iter().copied().cycle(),
                IDS[2].iter().copied().cycle(),
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
                        deck.increment(Id::new(number), part.into(), 1);
                        ids.push(number);
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

    fn card_data() -> &'static CardData {
        let mut data = Vec::new();

        for id in IDS[0].iter().chain(IDS[2].iter()) {
            data.push(Card {
                name: String::new(),
                ids: vec![Id::new(*id)],
                description: CardDescription::Regular(Vec::new()),
                search_text: String::new(),
                card_type: CardType::Spell(SpellType::Normal),
                limit: CardLimit::Unlimited,
                archetype: None,
            });
        }

        for id in &IDS[1] {
            data.push(Card {
                name: String::new(),
                ids: vec![Id::new(*id)],
                description: CardDescription::Regular(Vec::new()),
                search_text: String::new(),
                card_type: CardType::Monster {
                    race: Machine,
                    attribute: Dark,
                    stats: common::card::MonsterStats::Link {
                        atk: 0,
                        link_value: 0,
                        link_markers: LinkMarkers::default(),
                    },
                    effect: MonsterEffect::Normal,
                    is_tuner: false,
                },
                limit: CardLimit::Unlimited,
                archetype: None,
            });
        }

        Box::leak(Box::new(CardData::new(data)))
    }

    #[test]
    fn ydk_serialization() {
        for data in YdkData::get() {
            let mut output = Vec::new();
            save(&data.deck, card_data(), &mut output).unwrap();
            assert_eq!(data.ydk, String::from_utf8(output).unwrap());
        }
    }

    #[test]
    fn ydk_deserialization() {
        for data in YdkData::get() {
            let mut deck = load(&data.ydk, card_data()).unwrap();
            itertools::assert_equal(data.deck.entries(), deck.entries());

            // Ensure history is empty
            deck.undo();
            itertools::assert_equal(data.deck.entries(), deck.entries());
            deck.redo();
            itertools::assert_equal(data.deck.entries(), deck.entries());
        }
    }
}
