use std::fmt;

use common::card::{
    Attribute, Card, CardLimit, CardType, Id, LinkMarker, LinkMarkers, MonsterEffect, MonsterStats,
    MonsterType, Race, SpellType, TrapType,
};

use crate::ygoprodeck::{self, BanStatus};

#[derive(Debug, Clone)]
pub struct ProjectionError {
    card_id: u64,
    field: &'static str,
    error: ProjectionErrorKind,
}

impl ProjectionError {
    fn new_unexpected(card_id: u64, field: &'static str, value: &impl fmt::Debug) -> Self {
        Self {
            card_id,
            field,
            error: ProjectionErrorKind::UnexpectedValue(format!("{value:?}")),
        }
    }

    fn new_unknown(card_id: u64, field: &'static str, value: &impl fmt::Debug) -> Self {
        Self {
            card_id,
            field,
            error: ProjectionErrorKind::UnknownValue(format!("{value:?}")),
        }
    }
}

impl fmt::Display for ProjectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Error projecting field \"{}\" of card id {}: {}",
            self.field, self.card_id, self.error
        )
    }
}

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

trait TryUnwrapField<T> {
    fn try_unwrap_field(self, card_id: u64, field: &'static str) -> Result<T, ProjectionError>;
}

impl<T> TryUnwrapField<T> for Option<T> {
    fn try_unwrap_field(self, card_id: u64, field: &'static str) -> Result<T, ProjectionError> {
        self.ok_or(ProjectionError {
            card_id,
            field,
            error: ProjectionErrorKind::MissingField,
        })
    }
}

impl TryFrom<ygoprodeck::Card> for Card {
    type Error = ProjectionError;

    fn try_from(value: ygoprodeck::Card) -> Result<Self, Self::Error> {
        let card_type = CardType::try_from(&value)?;
        let limit = CardLimit::from(&value);

        Ok(Self {
            name: value.name,
            description: value.desc,
            card_type,
            limit,
            archetype: value.archetype,
        })
    }
}

impl TryFrom<&ygoprodeck::Card> for CardType {
    type Error = ProjectionError;

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

        type Src = ygoprodeck::CardType;
        match value.card_type {
            Src::EffectMonster
            | Src::PendulumEffectMonster
            | Src::PendulumEffectRitualMonster
            | Src::RitualEffectMonster
            | Src::FusionMonster
            | Src::LinkMonster
            | Src::PendulumEffectFusionMonster
            | Src::SynchroMonster
            | Src::SynchroPendulumEffectMonster
            | Src::XYZMonster
            | Src::XYZPendulumEffectMonster => {
                monster! {MonsterEffect::Effect}
            }
            Src::FlipEffectMonster | Src::PendulumFlipEffectMonster => {
                monster! {MonsterEffect::Flip}
            }
            Src::FlipTunerEffectMonster => {
                monster! {MonsterEffect::Flip, is_tuner: true}
            }
            Src::GeminiMonster => monster! {MonsterEffect::Gemini},
            Src::NormalMonster | Src::PendulumNormalMonster | Src::RitualMonster => {
                monster! {MonsterEffect::Normal}
            }
            Src::NormalTunerMonster => {
                monster! {MonsterEffect::Normal, is_tuner: true}
            }
            Src::PendulumTunerEffectMonster => monster! {MonsterEffect::Effect, is_tuner: true},
            Src::SpellCard => Ok(CardType::Spell(SpellType::try_from(value)?)),
            Src::SpiritMonster => monster! { MonsterEffect::Spirit},
            Src::ToonMonster => monster! { MonsterEffect::Toon},
            Src::TrapCard => Ok(CardType::Trap(TrapType::try_from(value)?)),
            Src::TunerMonster | Src::SynchroTunerMonster => {
                monster! { MonsterEffect::Effect, is_tuner: true}
            }
            Src::UnionEffectMonster => monster! { MonsterEffect::Union},
            Src::SkillCard | Src::Token => Err(ProjectionError::new_unexpected(
                value.id,
                "card_type",
                &value.card_type,
            )),
        }
    }
}

impl TryFrom<&ygoprodeck::Card> for Race {
    type Error = ProjectionError;

    fn try_from(value: &ygoprodeck::Card) -> Result<Self, Self::Error> {
        type Src = ygoprodeck::Race;

        let result = match value
            .race
            .as_ref()
            .try_unwrap_field(value.id, "race (monster)")?
        {
            Src::Aqua => Race::Aqua,
            Src::Beast => Race::Beast,
            Src::BeastWarrior => Race::BeastWarrior,
            Src::CreatorGod => Race::CreatorGod,
            Src::Cyberse => Race::Cyberse,
            Src::Dinosaur => Race::Dinosaur,
            Src::DivineBeast => Race::DivineBeast,
            Src::Dragon => Race::Dragon,
            Src::Fairy => Race::Fairy,
            Src::Fiend => Race::Fiend,
            Src::Fish => Race::Fish,
            Src::Illusion => Race::Illusion,
            Src::Insect => Race::Insect,
            Src::Machine => Race::Machine,
            Src::Plant => Race::Plant,
            Src::Psychic => Race::Psychic,
            Src::Pyro => Race::Pyro,
            Src::Reptile => Race::Reptile,
            Src::Rock => Race::Rock,
            Src::SeaSerpent => Race::SeaSerpent,
            Src::Spellcaster => Race::Spellcaster,
            Src::Thunder => Race::Thunder,
            Src::Warrior => Race::Warrior,
            Src::WingedBeast => Race::WingedBeast,
            Src::Wyrm => Race::Wyrm,
            Src::Zombie => Race::Zombie,
            race @ (Src::Normal
            | Src::Field
            | Src::Equip
            | Src::Continuous
            | Src::QuickPlay
            | Src::Ritual
            | Src::Counter) => {
                return Err(ProjectionError::new_unexpected(
                    value.id,
                    "race (monster)",
                    &race,
                ))
            }
            Src::Other(name) => {
                return Err(ProjectionError::new_unknown(
                    value.id,
                    "race (monster)",
                    &name,
                ))
            }
        };

        Ok(result)
    }
}

impl TryFrom<&ygoprodeck::Card> for Attribute {
    type Error = ProjectionError;

    fn try_from(value: &ygoprodeck::Card) -> Result<Self, Self::Error> {
        type Src = ygoprodeck::Attribute;

        let result = match value
            .attribute
            .as_ref()
            .try_unwrap_field(value.id, "attribute")?
        {
            Src::Dark => Attribute::Dark,
            Src::Earth => Attribute::Earth,
            Src::Fire => Attribute::Fire,
            Src::Light => Attribute::Light,
            Src::Water => Attribute::Water,
            Src::Wind => Attribute::Wind,
            Src::Divine => Attribute::Divine,
        };

        Ok(result)
    }
}

impl TryFrom<&ygoprodeck::Card> for MonsterStats {
    type Error = ProjectionError;

    fn try_from(value: &ygoprodeck::Card) -> Result<Self, Self::Error> {
        let atk = value.atk.try_unwrap_field(value.id, "atk stat")?;

        if matches!(value.card_type, ygoprodeck::CardType::LinkMonster) {
            Ok(MonsterStats::Link {
                atk,
                link_value: value.linkval.try_unwrap_field(value.id, "link value")?,
                link_markers: LinkMarkers::try_from(value)?,
            })
        } else {
            Ok(MonsterStats::Normal {
                atk,
                def: value.def.try_unwrap_field(value.id, "def stat")?,
                level: value.level.try_unwrap_field(value.id, "level")?,
                monster_type: Option::<MonsterType>::from(value),
                pendulum_scale: is_pendulum(value)
                    .then(|| value.scale.try_unwrap_field(value.id, "pendulum scale"))
                    .transpose()?,
            })
        }
    }
}

impl TryFrom<&ygoprodeck::Card> for LinkMarkers {
    type Error = ProjectionError;

    fn try_from(value: &ygoprodeck::Card) -> Result<Self, Self::Error> {
        let mut result = LinkMarkers::default();

        for marker in value
            .linkmarkers
            .as_ref()
            .try_unwrap_field(value.id, "link markers")?
        {
            result.add(LinkMarker::from(marker));
        }

        Ok(result)
    }
}

impl From<&ygoprodeck::LinkMarker> for LinkMarker {
    fn from(value: &ygoprodeck::LinkMarker) -> Self {
        type Src = ygoprodeck::LinkMarker;
        match value {
            Src::Top => LinkMarker::Top,
            Src::Bottom => LinkMarker::Bottom,
            Src::Left => LinkMarker::Left,
            Src::Right => LinkMarker::Right,
            Src::BottomLeft => LinkMarker::BottomLeft,
            Src::BottomRight => LinkMarker::BottomRight,
            Src::TopLeft => LinkMarker::TopLeft,
            Src::TopRight => LinkMarker::TopRight,
        }
    }
}

impl From<&ygoprodeck::Card> for Option<MonsterType> {
    fn from(value: &ygoprodeck::Card) -> Self {
        type Src = ygoprodeck::CardType;

        match value.card_type {
            Src::FusionMonster | Src::PendulumEffectFusionMonster => Some(MonsterType::Fusion),
            Src::SynchroMonster | Src::SynchroPendulumEffectMonster | Src::SynchroTunerMonster => {
                Some(MonsterType::Synchro)
            }
            Src::XYZMonster | Src::XYZPendulumEffectMonster => Some(MonsterType::Xyz),
            _ => None,
        }
    }
}

fn is_pendulum(card: &ygoprodeck::Card) -> bool {
    type Src = ygoprodeck::CardType;
    matches!(
        card.card_type,
        Src::PendulumEffectMonster
            | Src::PendulumEffectRitualMonster
            | Src::PendulumFlipEffectMonster
            | Src::PendulumNormalMonster
            | Src::PendulumTunerEffectMonster
            | Src::PendulumEffectFusionMonster
            | Src::SynchroPendulumEffectMonster
            | Src::XYZPendulumEffectMonster
    )
}

impl TryFrom<&ygoprodeck::Card> for SpellType {
    type Error = ProjectionError;

    fn try_from(value: &ygoprodeck::Card) -> Result<Self, Self::Error> {
        type Src = ygoprodeck::Race;
        let result = match value
            .race
            .as_ref()
            .try_unwrap_field(value.id, "race (spell)")?
        {
            Src::Normal => SpellType::Normal,
            Src::Field => SpellType::Field,
            Src::Equip => SpellType::Equip,
            Src::Continuous => SpellType::Continuous,
            Src::QuickPlay => SpellType::QuickPlay,
            Src::Ritual => SpellType::Ritual,
            race => {
                return Err(ProjectionError::new_unexpected(
                    value.id,
                    "race (spell)",
                    &race,
                ))
            }
        };

        Ok(result)
    }
}

impl TryFrom<&ygoprodeck::Card> for TrapType {
    type Error = ProjectionError;

    fn try_from(value: &ygoprodeck::Card) -> Result<Self, Self::Error> {
        type Src = ygoprodeck::Race;
        let result = match value
            .race
            .as_ref()
            .try_unwrap_field(value.id, "race (trap)")?
        {
            Src::Normal => TrapType::Normal,
            Src::Continuous => TrapType::Continuous,
            Src::Counter => TrapType::Counter,
            race => {
                return Err(ProjectionError::new_unexpected(
                    value.id,
                    "race (trap)",
                    &race,
                ))
            }
        };

        Ok(result)
    }
}

impl From<&ygoprodeck::Card> for CardLimit {
    fn from(value: &ygoprodeck::Card) -> Self {
        match value
            .banlist_info
            .as_ref()
            .and_then(|info| info.ban_tcg.as_ref())
        {
            None => CardLimit::Unlimited,
            Some(BanStatus::Limited) => CardLimit::Limited,
            Some(BanStatus::SemiLimited) => CardLimit::SemiLimited,
            Some(BanStatus::Banned) => CardLimit::Banned,
        }
    }
}

#[must_use]
pub fn project(card: ygoprodeck::Card) -> Option<(Id, Card)> {
    let id = Id::new(card.id);
    match card.try_into() {
        Ok(val) => Some((id, val)),
        Err(err) => {
            eprintln!("{err}");
            None
        }
    }
}
