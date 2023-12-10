use std::rc::Rc;

use common::card::{Card, CardData, CardType, Id, MonsterEffect, MonsterStats, MonsterType};
use leptos::{
    component, create_node_ref, create_signal, expect_context, html::Div, provide_context,
    use_context, view, IntoView, NodeRef, Show, SignalGet, SignalSet, WriteSignal,
};
use web_sys::MouseEvent;

use crate::ui::drag_drop::start_drag;

#[derive(Clone, Copy)]
struct TooltipData {
    card: &'static Card,
    node: NodeRef<Div>,
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
                    {&data.card.name}
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
            {(count > 1)
                .then(|| {
                    view! { <div class="count">{count}</div> }
                })}

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
