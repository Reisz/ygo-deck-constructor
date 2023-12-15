use std::rc::Rc;

use common::card::{
    Attribute, Card, CardData, CardType, Id, LinkMarker, MonsterEffect, MonsterStats, MonsterType,
    Race, SpellType, TrapType,
};
use leptos::{
    component, create_node_ref, create_signal, expect_context,
    html::{self, Div},
    provide_context, svg, use_context, view, IntoView, NodeRef, Show, SignalGet, SignalSet, View,
    WriteSignal,
};
use web_sys::MouseEvent;

use crate::ui::drag_drop::start_drag;

#[derive(Clone, Copy)]
struct TooltipData {
    card: &'static Card,
    node: NodeRef<Div>,
}

fn process_description(description: &'static str) -> Vec<View> {
    let mut result = Vec::new();
    let mut current_list: Option<Vec<&'static str>> = None;

    for paragraph in description.split('\n') {
        if let Some(paragraph) = paragraph.strip_prefix('●') {
            current_list.get_or_insert(Vec::default()).push(paragraph);
            continue;
        }

        if let Some(list) = current_list.take() {
            result.push(
                html::ul()
                    .child(
                        list.into_iter()
                            .map(|element| html::li().child(element))
                            .collect::<Vec<_>>(),
                    )
                    .into_view(),
            );
        }

        match paragraph.trim() {
            "[ Pendulum Effect ]" | "[ Monster Effect ]" => {
                result.push(
                    html::h2()
                        .child(paragraph.trim_matches(&['[', ']', ' ']))
                        .into_view(),
                );
            }
            paragraph => {
                result.push(html::p().child(paragraph).into_view());
            }
        }
    }

    result
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

fn get_tags(card_type: &CardType) -> Vec<View> {
    let mut tags = Vec::new();

    match card_type {
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
            tags.push(view! { <li>{monster} <span class="level">{*level}</span></li> });

            if let MonsterStats::Normal {
                pendulum_scale: Some(scale),
                ..
            } = stats
            {
                tags.push(view! { <li>"Pendulum" <span class="level">{*scale}</span></li> });
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
                tags.push(html::li().child(effect));
            }

            if *is_tuner {
                tags.push(html::li().child("Tuner"));
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
            tags.push(html::li().child(attribute));

            tags.push(html::li().child(map_race(*race)));
        }
        CardType::Trap(trap_type) => {
            let tag = match trap_type {
                TrapType::Normal => "Trap",
                TrapType::Counter => "Counter Trap",
                TrapType::Continuous => "Continuous Trap",
            };
            tags.push(html::li().child(tag));
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
            tags.push(html::li().child(tag));
        }
    }

    tags.into_iter()
        .map(IntoView::into_view)
        .intersperse_with(|| html::li().child("•").into_view())
        .collect()
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
            <span class="data">{*atk}</span>
        };

        let def = def.map(|def| {
            view! {
                <span class="label">"DEF"</span>
                <span class="data">{*def}</span>
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
                .class("stats", true)
                .child((atk, def, link))
                .into_view(),
        )
    } else {
        None
    }
}

#[component]
#[must_use]
pub fn CardTooltip() -> impl IntoView {
    let (tooltip_data, set_tooltip_data) = create_signal(None);
    provide_context(set_tooltip_data);

    let popup = move || {
        tooltip_data.get().map(|data: TooltipData| {
            let rect = data.node.get().unwrap().get_bounding_client_rect();
            let left = rect.right();
            let top = rect.top();
            view! {
                <div
                    class="card-tooltip"
                    style:left=format!("{left}px")
                    style:top=format!("{top}px")
                >
                    <h1>{&data.card.name}</h1>
                    <ul class="tags">{get_tags(&data.card.card_type)}</ul>
                    <Stats card_type=&data.card.card_type/>
                    <div class="description">{process_description(&data.card.description)}</div>
                </div>
            }
        })
    };

    view! { <Show when=move || tooltip_data.get().is_some()>{popup}</Show> }
}

#[component]
#[must_use]
pub fn CardView(
    id: Id,
    #[prop(default = 1)] count: usize,
    #[prop(optional)] on_delete: Option<Rc<dyn Fn(Id)>>,
) -> impl IntoView {
    let card = &use_context::<&'static CardData>().unwrap()[&id];

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
    let node = create_node_ref();
    view! {
        <div
            class=get_class(&card.card_type)
            ref=node
            draggable="true"
            on:dragstart=move |ev| start_drag(&ev, id, card)
            on:mouseover=move |_| set_tooltip_data.set(Some(TooltipData { card, node }))
            on:mouseout=move |_| set_tooltip_data.set(None)
            on:mouseup=on_click
            on:contextmenu=|ev| ev.prevent_default()
        >
            <img src=format!("images/{}.webp", id.get())/>
            {(count > 1).then(|| html::div().class("count", true).child(count))}
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
