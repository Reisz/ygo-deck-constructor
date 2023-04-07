use data::card::{
    Attribute, Card, CardLimit, CardType, Id, LinkMarker, LinkMarkers, MonsterEffect, MonsterStats,
    MonsterType, Race, SpellType, TrapType,
};

use crate::ygoprodeck::{self, BanStatus};

impl From<ygoprodeck::Card> for Card {
    fn from(value: ygoprodeck::Card) -> Self {
        let card_type = CardType::from(&value);
        let limit = CardLimit::from(&value);

        Self {
            name: value.name,
            description: value.desc,
            card_type,
            limit,
            archetype: value.archetype,
        }
    }
}

impl From<&ygoprodeck::Card> for CardType {
    fn from(value: &ygoprodeck::Card) -> Self {
        macro_rules! monster {
            ($effect:expr) => {
                monster! {$effect, is_tuner: false}
            };
            ($effect:expr, is_tuner: $tuner:expr) => {
                CardType::Monster {
                    race: Race::from(value),
                    attribute: Attribute::from(value),
                    stats: MonsterStats::from(value),
                    effect: $effect,
                    is_tuner: $tuner,
                }
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
            Src::SpellCard => CardType::Spell(SpellType::from(value)),
            Src::SpiritMonster => monster! { MonsterEffect::Spirit},
            Src::ToonMonster => monster! { MonsterEffect::Toon},
            Src::TrapCard => CardType::Trap(TrapType::from(value)),
            Src::TunerMonster | Src::SynchroTunerMonster => {
                monster! { MonsterEffect::Effect, is_tuner: true}
            }
            Src::UnionEffectMonster => monster! { MonsterEffect::Union},
            Src::SkillCard | Src::Token => panic!("Unexpected card type: {:?}", value.card_type),
        }
    }
}

impl From<&ygoprodeck::Card> for Race {
    fn from(value: &ygoprodeck::Card) -> Self {
        type Src = ygoprodeck::Race;

        match value.race.as_ref().expect("race") {
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
            race => panic!("Unexpected race: {race:?}"),
        }
    }
}

impl From<&ygoprodeck::Card> for Attribute {
    fn from(value: &ygoprodeck::Card) -> Self {
        type Src = ygoprodeck::Attribute;

        match value.attribute.as_ref().expect("attribute") {
            Src::Dark => Attribute::Dark,
            Src::Earth => Attribute::Earth,
            Src::Fire => Attribute::Fire,
            Src::Light => Attribute::Light,
            Src::Water => Attribute::Water,
            Src::Wind => Attribute::Wind,
            Src::Divine => Attribute::Divine,
        }
    }
}

impl From<&ygoprodeck::Card> for MonsterStats {
    fn from(value: &ygoprodeck::Card) -> Self {
        let atk = value.atk.expect("atk stat");

        if matches!(value.card_type, ygoprodeck::CardType::LinkMonster) {
            MonsterStats::Link {
                atk,
                link_value: value.linkval.expect("link value"),
                link_markers: LinkMarkers::from(value),
            }
        } else {
            MonsterStats::Normal {
                atk,
                def: value.def.expect("def stat"),
                level: value.level.expect("level"),
                monster_type: Option::<MonsterType>::from(value),
                pendulum_scale: is_pendulum(value).then(|| value.scale.expect("pendulum scale")),
            }
        }
    }
}

impl From<&ygoprodeck::Card> for LinkMarkers {
    fn from(value: &ygoprodeck::Card) -> Self {
        let mut result = LinkMarkers::default();
        for marker in value.linkmarkers.as_ref().expect("link markers") {
            result.add(LinkMarker::from(marker));
        }
        result
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

impl From<&ygoprodeck::Card> for SpellType {
    fn from(value: &ygoprodeck::Card) -> Self {
        type Src = ygoprodeck::Race;
        match value.race.as_ref().expect("race for spell card") {
            Src::Normal => SpellType::Normal,
            Src::Field => SpellType::Field,
            Src::Equip => SpellType::Equip,
            Src::Continuous => SpellType::Continuous,
            Src::QuickPlay => SpellType::QuickPlay,
            Src::Ritual => SpellType::Ritual,
            _ => panic!("Unexpected race for spell card: {:?}", value.race),
        }
    }
}

impl From<&ygoprodeck::Card> for TrapType {
    fn from(value: &ygoprodeck::Card) -> Self {
        type Src = ygoprodeck::Race;
        match value.race.as_ref().expect("race for trap card") {
            Src::Normal => TrapType::Normal,
            Src::Continuous => TrapType::Continuous,
            Src::Counter => TrapType::Counter,
            _ => panic!("Unexpected race for trap card: {:?}", value.race),
        }
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

pub fn project(card: ygoprodeck::Card) -> (Id, Card) {
    (Id::new(card.id), card.into())
}
