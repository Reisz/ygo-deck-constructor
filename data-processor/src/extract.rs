use common::card::{
    Attribute, CardLimit, CardPassword, CardType, CombatStat, FullCard, Header, LinkMarker,
    LinkMarkers, MonsterEffect, MonsterStats, MonsterType, Race, SpanKind, SpellType, TextBlock,
    TextPart, TrapType,
};
use log::warn;

use crate::{
    error::{ProcessingError, TryUnwrapField},
    ygoprodeck,
};

impl TryFrom<ygoprodeck::Card> for FullCard {
    type Error = ProcessingError;

    fn try_from(value: ygoprodeck::Card) -> Result<Self, Self::Error> {
        let description = (&value).into();
        let card_type = (&value).try_into()?;
        let limit = (&value).try_into()?;

        let name = value.name;
        let main_password = value.id;
        let all_passwords = value
            .card_images
            .into_iter()
            .map(|info| info.id)
            .collect::<Vec<CardPassword>>();
        let search_text = value.desc.to_lowercase();

        Ok(Self {
            name,
            main_password,
            all_passwords,
            description,
            search_text,
            card_type,
            limit,
        })
    }
}

impl From<&ygoprodeck::Card> for Vec<TextPart<String>> {
    fn from(card: &ygoprodeck::Card) -> Self {
        let mut in_list = false;
        card.desc
            .lines()
            .flat_map(|paragraph| {
                let mut result = vec![];

                // Lists
                if let Some(paragraph) = paragraph.strip_prefix('●') {
                    if !in_list {
                        result.push(TextPart::Block(TextBlock::List));
                        in_list = true;
                    }

                    result.push(TextPart::Block(TextBlock::ListEntry));
                    result.push(TextPart::Span(SpanKind::Normal, paragraph.to_owned()));
                    result.push(TextPart::EndBlock(TextBlock::ListEntry));

                    return result;
                }
                if in_list {
                    result.push(TextPart::EndBlock(TextBlock::List));
                    in_list = false;
                }

                // Headers
                match paragraph.trim() {
                    "[ Pendulum Effect ]" => {
                        if !is_pendulum(card) {
                            warn!(
                                "{}",
                                ProcessingError::new_unexpected(
                                    card.id,
                                    "description",
                                    "pendulum header on non-pendulum card",
                                )
                            );
                        }

                        result.push(TextPart::Header(Header::PendulumEffect));
                        return result;
                    }
                    "[ Monster Effect ]" => {
                        result.push(TextPart::Header(Header::MonsterEffect));
                        return result;
                    }
                    _ => {}
                }

                result.extend_from_slice(&[
                    TextPart::Block(TextBlock::Paragraph),
                    TextPart::Span(SpanKind::Normal, paragraph.to_owned()),
                    TextPart::EndBlock(TextBlock::Paragraph),
                ]);

                result
            })
            .collect()
    }
}

impl TryFrom<&ygoprodeck::Card> for CardType {
    type Error = ProcessingError;

    fn try_from(value: &ygoprodeck::Card) -> Result<Self, Self::Error> {
        macro_rules! monster {
            ($effect:expr) => {
                monster! {$effect, is_tuner: false}
            };
            ($effect:expr, is_tuner: $tuner:expr) => {
                Ok(CardType::Monster {
                    race: Race::try_from(value)?,
                    attribute: Attribute::try_from(value)?,
                    stats: MonsterStats::try_from(value)?,
                    effect: $effect,
                    is_tuner: $tuner,
                })
            };
        }

        match value.card_type.as_str() {
            "Effect Monster"
            | "Pendulum Effect Monster"
            | "Pendulum Effect Ritual Monster"
            | "Ritual Effect Monster"
            | "Fusion Monster"
            | "Link Monster"
            | "Pendulum Effect Fusion Monster"
            | "Synchro Monster"
            | "Synchro Pendulum Effect Monster"
            | "XYZ Monster"
            | "XYZ Pendulum Effect Monster" => {
                monster! {MonsterEffect::Effect}
            }
            "Flip Effect Monster" | "Pendulum Flip Effect Monster" => {
                monster! {MonsterEffect::Flip}
            }
            "Flip Tuner Effect Monster" => {
                monster! {MonsterEffect::Flip, is_tuner: true}
            }
            "Gemini Monster" => monster! {MonsterEffect::Gemini},
            "Normal Monster" | "Pendulum Normal Monster" | "Ritual Monster" => {
                monster! {MonsterEffect::Normal}
            }
            "Normal Tuner Monster" => {
                monster! {MonsterEffect::Normal, is_tuner: true}
            }
            "Pendulum Tuner Effect Monster" => monster! {MonsterEffect::Effect, is_tuner: true},
            "Spell Card" => Ok(CardType::Spell(SpellType::try_from(value)?)),
            "Spirit Monster" => monster! { MonsterEffect::Spirit},
            "Toon Monster" => monster! { MonsterEffect::Toon},
            "Trap Card" => Ok(CardType::Trap(TrapType::try_from(value)?)),
            "Tuner Monster" | "Synchro Tuner Monster" => {
                monster! { MonsterEffect::Effect, is_tuner: true}
            }
            "Union Effect Monster" => monster! { MonsterEffect::Union},
            name => Err(ProcessingError::new_unexpected(
                value.id,
                "card_type",
                &name,
            )),
        }
    }
}

impl TryFrom<&ygoprodeck::Card> for Race {
    type Error = ProcessingError;

    fn try_from(value: &ygoprodeck::Card) -> Result<Self, Self::Error> {
        let result = match value
            .race
            .as_deref()
            .try_unwrap_field(value.id, "race (monster)")?
        {
            "Aqua" => Race::Aqua,
            "Beast" => Race::Beast,
            "Beast-Warrior" => Race::BeastWarrior,
            "Creator-God" | "Creator God" => Race::CreatorGod,
            "Cyberse" => Race::Cyberse,
            "Dinosaur" => Race::Dinosaur,
            "Divine-Beast" => Race::DivineBeast,
            "Dragon" => Race::Dragon,
            "Fairy" => Race::Fairy,
            "Fiend" => Race::Fiend,
            "Fish" => Race::Fish,
            "Illusion" => Race::Illusion,
            "Insect" => Race::Insect,
            "Machine" => Race::Machine,
            "Plant" => Race::Plant,
            "Psychic" => Race::Psychic,
            "Pyro" => Race::Pyro,
            "Reptile" => Race::Reptile,
            "Rock" => Race::Rock,
            "Sea Serpent" => Race::SeaSerpent,
            "Spellcaster" => Race::Spellcaster,
            "Thunder" => Race::Thunder,
            "Warrior" => Race::Warrior,
            "Winged Beast" => Race::WingedBeast,
            "Wyrm" => Race::Wyrm,
            "Zombie" => Race::Zombie,
            name => {
                return Err(ProcessingError::new_unknown(
                    value.id,
                    "race (monster)",
                    &name,
                ));
            }
        };

        Ok(result)
    }
}

impl TryFrom<&ygoprodeck::Card> for Attribute {
    type Error = ProcessingError;

    fn try_from(value: &ygoprodeck::Card) -> Result<Self, Self::Error> {
        let result = match value
            .attribute
            .as_deref()
            .try_unwrap_field(value.id, "attribute")?
        {
            "DARK" => Attribute::Dark,
            "EARTH" => Attribute::Earth,
            "FIRE" => Attribute::Fire,
            "LIGHT" => Attribute::Light,
            "WATER" => Attribute::Water,
            "WIND" => Attribute::Wind,
            "DIVINE" => Attribute::Divine,
            name => return Err(ProcessingError::new_unknown(value.id, "attribute", &name)),
        };

        Ok(result)
    }
}

impl TryFrom<&ygoprodeck::Card> for MonsterStats {
    type Error = ProcessingError;

    fn try_from(value: &ygoprodeck::Card) -> Result<Self, Self::Error> {
        let atk = value.atk.try_unwrap_field(value.id, "atk stat")?;
        let atk = to_combat_stat(atk, value.id, "atk stat")?;

        if value.card_type == "Link Monster" {
            Ok(MonsterStats::Link {
                atk,
                link_value: value.linkval.try_unwrap_field(value.id, "link value")?,
                link_markers: LinkMarkers::try_from(value)?,
            })
        } else {
            let def = value.def.try_unwrap_field(value.id, "def stat")?;
            let def = to_combat_stat(def, value.id, "def stat")?;

            Ok(MonsterStats::Normal {
                atk,
                def,
                level: value.level.try_unwrap_field(value.id, "level")?,
                monster_type: Option::<MonsterType>::from(value),
                pendulum_scale: is_pendulum(value)
                    .then(|| value.scale.try_unwrap_field(value.id, "pendulum scale"))
                    .transpose()?,
            })
        }
    }
}

fn to_combat_stat(
    value: i16,
    password: CardPassword,
    field: &'static str,
) -> Result<CombatStat, ProcessingError> {
    if value == -1 {
        return Ok(CombatStat::questionmark());
    }

    if !(0..=5000).contains(&value) {
        return Err(ProcessingError::new_unexpected(password, field, &value));
    }

    let value = u16::try_from(value).unwrap();
    Ok(CombatStat::new(value))
}

impl TryFrom<&ygoprodeck::Card> for LinkMarkers {
    type Error = ProcessingError;

    fn try_from(value: &ygoprodeck::Card) -> Result<Self, Self::Error> {
        let mut result = LinkMarkers::default();

        for marker in value
            .linkmarkers
            .as_ref()
            .try_unwrap_field(value.id, "link markers")?
        {
            result.add(to_link_marker(marker, value.id)?);
        }

        Ok(result)
    }
}

fn to_link_marker(value: &str, password: CardPassword) -> Result<LinkMarker, ProcessingError> {
    Ok(match value {
        "Top" => LinkMarker::Top,
        "Bottom" => LinkMarker::Bottom,
        "Left" => LinkMarker::Left,
        "Right" => LinkMarker::Right,
        "Bottom-Left" => LinkMarker::BottomLeft,
        "Bottom-Right" => LinkMarker::BottomRight,
        "Top-Left" => LinkMarker::TopLeft,
        "Top-Right" => LinkMarker::TopRight,
        name => {
            return Err(ProcessingError::new_unexpected(
                password,
                "link marker",
                &name,
            ));
        }
    })
}

impl From<&ygoprodeck::Card> for Option<MonsterType> {
    fn from(value: &ygoprodeck::Card) -> Self {
        if value.card_type.contains("Ritual") {
            Some(MonsterType::Ritual)
        } else if value.card_type.contains("Fusion") {
            Some(MonsterType::Fusion)
        } else if value.card_type.contains("Synchro") {
            Some(MonsterType::Synchro)
        } else if value.card_type.contains("XYZ") {
            Some(MonsterType::Xyz)
        } else {
            None
        }
    }
}

fn is_pendulum(card: &ygoprodeck::Card) -> bool {
    card.card_type.contains("Pendulum")
}

impl TryFrom<&ygoprodeck::Card> for SpellType {
    type Error = ProcessingError;

    fn try_from(value: &ygoprodeck::Card) -> Result<Self, Self::Error> {
        let result = match value
            .race
            .as_deref()
            .try_unwrap_field(value.id, "race (spell)")?
        {
            "Normal" => SpellType::Normal,
            "Field" => SpellType::Field,
            "Equip" => SpellType::Equip,
            "Continuous" => SpellType::Continuous,
            "Quick-Play" => SpellType::QuickPlay,
            "Ritual" => SpellType::Ritual,
            race => {
                return Err(ProcessingError::new_unexpected(
                    value.id,
                    "race (spell)",
                    &race,
                ));
            }
        };

        Ok(result)
    }
}

impl TryFrom<&ygoprodeck::Card> for TrapType {
    type Error = ProcessingError;

    fn try_from(value: &ygoprodeck::Card) -> Result<Self, Self::Error> {
        let result = match value
            .race
            .as_deref()
            .try_unwrap_field(value.id, "race (trap)")?
        {
            "Normal" => TrapType::Normal,
            "Continuous" => TrapType::Continuous,
            "Counter" => TrapType::Counter,
            race => {
                return Err(ProcessingError::new_unexpected(
                    value.id,
                    "race (trap)",
                    &race,
                ));
            }
        };

        Ok(result)
    }
}

impl TryFrom<&ygoprodeck::Card> for CardLimit {
    type Error = ProcessingError;

    fn try_from(value: &ygoprodeck::Card) -> Result<Self, Self::Error> {
        Ok(
            match value
                .banlist_info
                .as_ref()
                .and_then(|info| info.ban_tcg.as_deref())
            {
                None => CardLimit::Unlimited,
                Some("Limited") => CardLimit::Limited,
                Some("Semi-Limited") => CardLimit::SemiLimited,
                Some("Forbidden") => CardLimit::Forbidden,
                Some(name) => {
                    return Err(ProcessingError::new_unexpected(
                        value.id,
                        "ban status",
                        name,
                    ));
                }
            },
        )
    }
}
