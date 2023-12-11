use std::rc::Rc;

use common::card::{Card, CardData, CardType, Id, MonsterEffect, MonsterStats, MonsterType};
use leptos::{
    component, create_node_ref, create_signal, expect_context,
    html::{self, Div},
    provide_context, use_context, view, IntoView, NodeRef, Show, SignalGet, SignalSet, View,
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
