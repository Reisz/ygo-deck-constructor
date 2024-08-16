use std::rc::Rc;

use common::{
    card::{
        Attribute, Card, CardDescription, CardDescriptionPart, CardType, LinkMarker, MonsterEffect,
        MonsterStats, MonsterType, Race, SpellType, TrapType,
    },
    card_data::{CardData, Id},
    transfer::{IMAGE_DIRECTORY, IMAGE_FILE_ENDING},
};
use itertools::intersperse_with;
use leptos::{
    component, create_node_ref, create_signal, expect_context,
    html::{self, Div},
    provide_context, svg, view, CollectView, IntoView, NodeRef, Show, SignalGet, SignalSet, View,
    WriteSignal,
};
use web_sys::MouseEvent;

use crate::ui::drag_drop::start_drag;

#[derive(Clone, Copy)]
struct TooltipData {
    card: &'static Card,
    node: NodeRef<Div>,
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

    intersperse_with(tags.into_iter().map(IntoView::into_view), || {
        html::li().child("•").into_view()
    })
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
fn DescriptionParts(parts: &'static [CardDescriptionPart]) -> impl IntoView {
    parts
        .iter()
        .map(|part| match part {
            CardDescriptionPart::Paragraph(paragraph) => html::p().child(paragraph).into_view(),
            CardDescriptionPart::List(elements) => html::ul()
                .child(
                    elements
                        .iter()
                        .map(|element| html::li().child(element))
                        .collect_view(),
                )
                .into_view(),
        })
        .collect::<Vec<_>>()
}

#[component]
#[must_use]
fn Description(description: &'static CardDescription) -> impl IntoView {
    match description {
        CardDescription::Regular(elements) => {
            view! { <DescriptionParts parts=elements.as_slice() /> }.into_view()
        }
        CardDescription::Pendulum {
            spell_effect,
            monster_effect,
        } => view! {
            <h2>"Spell Effect"</h2>
            <DescriptionParts parts=spell_effect.as_slice() />
            <h2>"Monster Effect"</h2>
            <DescriptionParts parts=monster_effect />
        }
        .into_view(),
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
                    style=("--left", format!("{left}px"))
                    style:top=format!("{top}px")
                >
                    <h1>{data.card.name}</h1>
                    <ul class="tags">{get_tags(&data.card.card_type)}</ul>
                    <Stats card_type=&data.card.card_type />
                    <Description description=&data.card.description />
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
    let card = &expect_context::<&'static CardData>()[id];
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
    let node = create_node_ref();
    view! {
        <div
            class=get_class(&card.card_type)
            ref=node
            draggable="true"
            on:dragstart=move |ev| start_drag(&ev, card)
            on:mouseover=move |_| set_tooltip_data.set(Some(TooltipData { card, node }))
            on:mouseout=move |_| set_tooltip_data.set(None)
            on:mouseup=on_click
            on:contextmenu=|ev| ev.prevent_default()
        >
            <img src=format!("{IMAGE_DIRECTORY}/{password}.{IMAGE_FILE_ENDING}") />
            {(count > 1)
                .then(|| html::div().class("count", true).class("backdrop", true).child(count))}
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
