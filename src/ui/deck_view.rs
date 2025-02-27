use std::sync::Arc;

use common::{
    card_data::CardData,
    deck_part::{DeckPart, EntriesForPart},
};
use leptos::prelude::*;

use crate::{
    deck::Deck,
    deck_order::deck_order,
    ui::{
        card_view::CardView,
        drag_drop::{DragInfo, DropEffect, get_drag_info, get_dropped_card, set_drop_effect},
    },
};

#[component]
fn PartView(part: DeckPart) -> impl IntoView {
    let deck = expect_context::<RwSignal<Deck>>();
    let cards = expect_context::<CardData>();

    let delete = move |delete_id| {
        deck.update(|deck| {
            deck.decrement(delete_id, part.into(), 1);
        });
    };
    let delete = Arc::new(delete);

    let drag_over = move |ev| {
        let drag_info = get_drag_info(&ev);

        let ok = match part {
            DeckPart::Main => matches!(drag_info, DragInfo::MainCard),
            DeckPart::Extra => matches!(drag_info, DragInfo::ExtraCard),
            DeckPart::Side => matches!(drag_info, DragInfo::MainCard | DragInfo::ExtraCard),
        };

        if ok {
            set_drop_effect(&ev, DropEffect::Copy);
            ev.prevent_default();
        }
    };

    let entries = Memo::new(move |_| {
        let mut result =
            deck.with(|deck| deck.entries().for_part(part, &cards).collect::<Vec<_>>());
        result.sort_unstable_by(move |(lhs, _), (rhs, _)| deck_order(&cards[*lhs], &cards[*rhs]));
        result
    });

    view! {
        <h2>{part.to_string()}</h2>
        <div class="part-size">
            <span class="current">
                {move || {
                    deck.with(|deck| {
                        deck.entries().for_part(part, &cards).map(|(_, count)| count).sum::<u8>()
                    })
                }}

            </span>
            <span class="divider">" / "</span>
            <span class="max">{part.max()}</span>
        </div>
        <div
            class="card-list"
            on:dragenter=drag_over
            on:dragover=drag_over
            on:drop=move |ev| {
                let id = get_dropped_card(&ev, &cards);
                deck.update(|deck| {
                    deck.increment(id, part.into(), 1);
                });
            }
        >

            <For
                each=move || entries.get()
                key=|el| *el
                children=move |(id, count)| {
                    let delete = delete.clone();
                    view! { <CardView id=id count=count on_delete=delete /> }
                }
            />

        </div>
    }
}

#[component]
#[must_use]
pub fn DeckView() -> impl IntoView {
    view! {
        <div class="deck-view">
            <PartView part=DeckPart::Main />
            <PartView part=DeckPart::Extra />
            <PartView part=DeckPart::Side />
        </div>
    }
}
