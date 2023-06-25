use std::rc::Rc;

use common::card::{CardData, CardType, Id, MonsterEffect, MonsterStats, MonsterType};
use leptos::{component, use_context, view, IntoView, Scope};
use web_sys::MouseEvent;
use ygo_deck_constructor::drag_drop::start_drag;

#[component]
pub fn CardView(
    cx: Scope,
    id: Id,
    #[prop(optional)] on_delete: Option<Rc<dyn Fn(Id)>>,
) -> impl IntoView {
    let card = &use_context::<&'static CardData>(cx).unwrap()[&id];

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

    view! { cx,
        <div
            class=get_class(&card.card_type)
            draggable="true"
            on:dragstart=move |ev| start_drag(&ev, id, card)
            on:mouseup=on_click
            on:contextmenu=|ev| ev.prevent_default()
        >
            <img src=format!("images/{}.webp", id.get())/>
            <div class="tooltip">{&card.name}</div>
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
