//! Order by color, then "impact", then name

use std::cmp::Ordering;

use common::card::{Card, CardType, MonsterEffect, MonsterStats, MonsterType, SpellType, TrapType};

const fn spell_index(spell_type: SpellType) -> u32 {
    match spell_type {
        SpellType::Field => 5,
        SpellType::Ritual => 4,
        SpellType::Continuous => 3,
        SpellType::Equip => 2,
        SpellType::QuickPlay => 1,
        SpellType::Normal => 0,
    }
}

const fn trap_index(trap_type: TrapType) -> u32 {
    match trap_type {
        TrapType::Continuous => 2,
        TrapType::Counter => 1,
        TrapType::Normal => 0,
    }
}

const fn monster_index(monster_type: Option<MonsterType>) -> u32 {
    match monster_type {
        None => 4,
        Some(MonsterType::Ritual) => 3,
        Some(MonsterType::Fusion) => 2,
        Some(MonsterType::Synchro) => 1,
        Some(MonsterType::Xyz) => 0,
    }
}

fn monster_stats_indices(stats: &MonsterStats) -> Vec<u32> {
    let mut result = Vec::new();

    match stats {
        MonsterStats::Normal {
            level,
            monster_type,
            pendulum_scale,
            ..
        } => {
            result.push(1);
            result.push(monster_index(*monster_type));

            match pendulum_scale {
                None => result.push(1),
                Some(scale) => {
                    result.push(0);
                    result.push((*scale).into());
                }
            }

            result.push((*level).into());
        }
        MonsterStats::Link { link_value, .. } => {
            result.push(0);
            result.push((*link_value).into());
        }
    }

    result
}

fn type_indices(card_type: &CardType) -> Vec<u32> {
    let mut result = Vec::new();

    match card_type {
        CardType::Monster {
            stats,
            effect: MonsterEffect::Normal,
            ..
        } => {
            result.push(3);
            result.append(&mut monster_stats_indices(stats));
        }
        CardType::Monster { stats, .. } => {
            result.push(2);
            result.append(&mut monster_stats_indices(stats));
        }
        CardType::Spell(spell_type) => {
            result.push(1);
            result.push(spell_index(*spell_type));
        }
        CardType::Trap(trap_type) => {
            result.push(0);
            result.push(trap_index(*trap_type));
        }
    };

    result
}

#[must_use]
pub fn deck_order(lhs: &Card, rhs: &Card) -> Ordering {
    Ordering::Equal
        .then(
            type_indices(&lhs.card_type)
                .cmp(&type_indices(&rhs.card_type))
                .reverse(),
        )
        .then(lhs.name.cmp(&rhs.name))
}
