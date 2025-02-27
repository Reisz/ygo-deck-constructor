use std::sync::Arc;

use common::{
    card::{
        Attribute, Card, CardType, LinkMarker, MonsterEffect, MonsterStats, MonsterType, Race,
        SpanKind, SpellType, TextBlock, TextPart, TrapType,
    },
    card_data::{CardData, Id},
    transfer::{IMAGE_DIRECTORY, IMAGE_FILE_ENDING},
};
use itertools::intersperse_with;
use leptos::{
    html::{self, ElementChild},
    prelude::*,
    svg,
};
use web_sys::MouseEvent;

use crate::ui::drag_drop::start_drag;

#[derive(Clone, Copy)]
struct TooltipData {
    card: &'static Card,
    node: NodeRef<html::Div>,
}

fn map_race(race: Race) -> &'static str {
    match race {
        Race::Aqua => "Aqua",
        Race::Beast => "Beast",
        Race::BeastWarrior => "Beast-Warrior",
        Race::CreatorGod => "Creator God",
        Race::Cyberse => "Cyberse",
        Race::Dinosaur => "Dinosaur",
        Race::DivineBeast => "Divine-Beast",
        Race::Dragon => "Dragon",
        Race::Fairy => "Fairy",
        Race::Fiend => "Fiend",
        Race::Fish => "Fish",
        Race::Illusion => "Illusion",
        Race::Insect => "Insect",
        Race::Machine => "Machine",
        Race::Plant => "Plant",
        Race::Psychic => "Psychic",
        Race::Pyro => "Pyro",
        Race::Reptile => "Reptile",
        Race::Rock => "Rock",
        Race::SeaSerpent => "Sea Serpent",
        Race::Spellcaster => "Spellcaster",
        Race::Thunder => "Thunder",
        Race::Warrior => "Warrior",
        Race::WingedBeast => "Winged Beast",
        Race::Wyrm => "Wyrm",
        Race::Zombie => "Zombie",
    }
}

fn get_tags(card: &Card) -> Vec<AnyView> {
    let mut tags = Vec::new();

    match &card.card_type {
        CardType::Monster {
            race,
            attribute,
            stats,
            effect,
            is_tuner,
        } => {
            let (monster, level) = match stats {
                MonsterStats::Normal {
                    level,
                    monster_type,
                    ..
                } => {
                    let monster_type = match monster_type {
                        None => "Monster",
                        Some(MonsterType::Fusion) => "Fusion Monster",
                        Some(MonsterType::Ritual) => "Ritual Monster",
                        Some(MonsterType::Synchro) => "Synchro Monster",
                        Some(MonsterType::Xyz) => "Xyz Monster",
                    };
                    (monster_type, level)
                }
                MonsterStats::Link { link_value, .. } => ("Link Monster", link_value),
            };
            tags.push(view! { <li>{monster} <span class="level">{*level}</span></li> }.into_any());

            if let MonsterStats::Normal {
                pendulum_scale: Some(scale),
                ..
            } = stats
            {
                tags.push(
                    view! { <li>"Pendulum" <span class="level">{*scale}</span></li> }.into_any(),
                );
            }

            let effect = match effect {
                MonsterEffect::Normal => None,
                MonsterEffect::Effect => Some("Effect"),
                MonsterEffect::Spirit => Some("Spirit Effect"),
                MonsterEffect::Toon => Some("Toon Effect"),
                MonsterEffect::Union => Some("Union Effect"),
                MonsterEffect::Gemini => Some("Gemini Effect"),
                MonsterEffect::Flip => Some("Flip Effect"),
            };
            if let Some(effect) = effect {
                tags.push(html::li().child(effect).into_any());
            }

            if *is_tuner {
                tags.push(html::li().child("Tuner").into_any());
            }

            let attribute = match attribute {
                Attribute::Dark => "Dark",
                Attribute::Earth => "Earth",
                Attribute::Fire => "Fire",
                Attribute::Light => "Light",
                Attribute::Water => "Water",
                Attribute::Wind => "Wind",
                Attribute::Divine => "Divine",
            };
            tags.push(html::li().child(attribute).into_any());

            tags.push(html::li().child(map_race(*race)).into_any());
        }
        CardType::Trap(trap_type) => {
            let tag = match trap_type {
                TrapType::Normal => "Trap",
                TrapType::Counter => "Counter Trap",
                TrapType::Continuous => "Continuous Trap",
            };
            tags.push(html::li().child(tag).into_any());
        }
        CardType::Spell(spell_type) => {
            let tag = match spell_type {
                SpellType::Normal => "Spell",
                SpellType::Field => "Field Spell",
                SpellType::Equip => "Equip Spell",
                SpellType::Continuous => "Continuous Spell",
                SpellType::QuickPlay => "Quick-Play Spell",
                SpellType::Ritual => "Ritual Spell",
            };
            tags.push(html::li().child(tag).into_any());
        }
    }

    let limit = match card.limit {
        common::card::CardLimit::Unlimited => "Unlimited",
        common::card::CardLimit::SemiLimited => "Semi-Limited",
        common::card::CardLimit::Limited => "Limited",
        common::card::CardLimit::Forbidden => "Forbidden",
    };
    tags.push(html::li().child(limit).into_any());

    intersperse_with(tags, || html::li().child("•").into_any()).collect()
}

fn link_marker_path(link_marker: LinkMarker) -> &'static str {
    // These use a 6 x 6 grid
    match link_marker {
        LinkMarker::Top => "3,0 4,2 2,2",
        LinkMarker::TopLeft => "0,0 2,0 0,2",
        LinkMarker::TopRight => "4,0 6,0 6,2",
        LinkMarker::Right => "4,2 6,3 4,4",
        LinkMarker::BottomRight => "6,4 6,6 4,6",
        LinkMarker::Bottom => "3,6 4,4 2,4",
        LinkMarker::BottomLeft => "0,4 0,6 2,6",
        LinkMarker::Left => "0,3 2,2 2,4",
    }
}

#[component]
#[must_use]
fn Stats(card_type: &'static CardType) -> impl IntoView {
    if let CardType::Monster { stats, .. } = &card_type {
        let (atk, def, link) = match stats {
            MonsterStats::Normal { atk, def, .. } => (atk, Some(def), None),
            MonsterStats::Link {
                atk, link_markers, ..
            } => (atk, None, Some(link_markers)),
        };

        let atk = view! {
            <span class="label">"ATK"</span>
            <span class="data">{atk.to_string()}</span>
        };

        let def = def.map(|def| {
            view! {
                <span class="label">"DEF"</span>
                <span class="data">{def.to_string()}</span>
            }
        });

        let link = link.map(|link| {
            let markers = LinkMarker::iter()
                .map(|marker| {
                    svg::polygon()
                        .attr("points", link_marker_path(marker))
                        .attr("fill-opacity", if link.has(marker) { "1" } else { "0.2" })
                })
                .collect::<Vec<_>>();
            view! {
                <span class="label">"LINKS"</span>
                <svg class="link" viewBox="0 0 6 6" width="1em" height="1em">
                    {markers}
                </svg>
            }
        });

        Some(
            html::div()
                .class("stats")
                .child((atk, def, link))
                .into_view(),
        )
    } else {
        None
    }
}

#[component]
#[must_use]
fn DescriptionParts(parts: &'static [TextPart<&'static str>]) -> impl IntoView {
    let mut div = Vec::new();
    let mut current_list = None;
    let mut current_block = None;

    for part in parts {
        match part {
            TextPart::Block(block) => match block {
                TextBlock::List => current_list = Some(Vec::new()),
                TextBlock::ListEntry | TextBlock::Paragraph => current_block = Some(Vec::new()),
            },
            TextPart::EndBlock(block) => match block {
                TextBlock::Paragraph => {
                    div.push(html::p().child(current_block.take().unwrap()).into_any());
                }
                TextBlock::List => {
                    div.push(html::ul().child(current_list.take().unwrap()).into_any());
                }
                TextBlock::ListEntry => {
                    current_list
                        .as_mut()
                        .unwrap()
                        .push(html::li().child(current_block.take().unwrap()).into_any());
                }
            },
            TextPart::Header(header) => {
                let text = match header {
                    common::card::Header::PendulumEffect => "Pendulum Effect",
                    common::card::Header::MonsterEffect => "Monster Effect",
                };
                div.push(html::h2().child(text).into_any());
            }
            TextPart::Span(kind, text) => match kind {
                SpanKind::Normal => {
                    current_block.as_mut().unwrap().push(text.into_any());
                }
            },
        };
    }

    html::div().child(div)
}

#[component]
#[must_use]
pub fn CardTooltip() -> impl IntoView {
    let (tooltip_data, set_tooltip_data) = signal(None);
    provide_context(set_tooltip_data);

    let popup = move || {
        tooltip_data.get().map(|data: TooltipData| {
            let rect = data.node.get().unwrap().get_bounding_client_rect();
            let left = rect.right();
            let top = rect.top();
            view! {
                <div
                    class="card-tooltip"
                    style=("--left", format!("{left}px"))
                    style:top=format!("{top}px")
                >
                    <h1>{data.card.name}</h1>
                    <ul class="tags">{get_tags(data.card)}</ul>
                    <Stats card_type=&data.card.card_type />
                    <DescriptionParts parts=data.card.description />
                </div>
            }
        })
    };

    view! { <Show when=move || tooltip_data.get().is_some()>{popup}</Show> }
}

#[component]
pub(crate) fn CardView(
    id: Id,
    #[prop(default = 1)] count: u8,
    #[prop(optional)] on_delete: Option<Arc<dyn Fn(Id)>>,
) -> impl IntoView {
    let card = expect_context::<CardData>().get(id);
    let password = card.password;

    let on_click: Box<dyn FnMut(MouseEvent)> = if let Some(on_delete) = on_delete {
        Box::new(move |ev: MouseEvent| {
            if ev.button() == 2 {
                on_delete(id);
                ev.prevent_default();
            }
        })
    } else {
        Box::new(move |_ev: MouseEvent| {})
    };

    let set_tooltip_data = expect_context::<WriteSignal<Option<TooltipData>>>();
    let node = NodeRef::new();
    view! {
        <div
            class=get_class(&card.card_type)
            node_ref=node
            draggable="true"
            on:dragstart=move |ev| start_drag(&ev, card)
            on:mouseover=move |_| set_tooltip_data.set(Some(TooltipData { card, node }))
            on:mouseout=move |_| set_tooltip_data.set(None)
            on:mouseup=on_click
            on:contextmenu=|ev| ev.prevent_default()
        >
            <img src=format!("{IMAGE_DIRECTORY}/{password}.{IMAGE_FILE_ENDING}") />
            {(count > 1).then(|| html::div().class("count backdrop").child(count))}
            {(count > card.limit.count()).then(|| html::div().class("error backdrop").child("!"))}
        </div>
    }
}

fn get_class(card_type: &CardType) -> String {
    let mut classes = vec!["card"];

    classes.push(match card_type {
        CardType::Monster { .. } => "monster",
        CardType::Spell(..) => "spell",
        CardType::Trap(..) => "trap",
    });

    if let CardType::Monster { stats, effect, .. } = card_type {
        if !matches!(effect, MonsterEffect::Normal) {
            classes.push("effect");
        }

        if matches!(stats, MonsterStats::Link { .. }) {
            classes.push("link");
        }

        if let MonsterStats::Normal {
            monster_type: Some(monster_type),
            ..
        } = stats
        {
            classes.push(match monster_type {
                MonsterType::Ritual => "ritual",
                MonsterType::Fusion => "fusion",
                MonsterType::Synchro => "synchro",
                MonsterType::Xyz => "xyz",
            });
        }

        if matches!(
            stats,
            MonsterStats::Normal {
                pendulum_scale: Some(_),
                ..
            }
        ) {
            classes.push("pendulum");
        }
    }

    classes.join(" ")
}
