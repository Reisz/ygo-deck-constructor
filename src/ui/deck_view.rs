use std::rc::Rc;

use common::{card::Id, card_data::CardData};
use leptos::{component, expect_context, view, For, IntoView, RwSignal, SignalUpdate, SignalWith};

use crate::{
    deck::Deck,
    deck_part::DeckPart,
    ui::{
        card_view::CardView,
        drag_drop::{get_dragged_card, set_drop_effect, DropEffect},
    },
};

#[component]
fn PartView(part: DeckPart) -> impl IntoView {
    let deck = expect_context::<RwSignal<Deck>>();
    let cards = expect_context::<&'static CardData>();

    let delete = move |delete_id| {
        deck.update(|deck| {
            deck.decrement(delete_id, part.into(), 1);
        });
    };
    let delete = Rc::new(delete);

    let drag_over = move |ev| {
        if let Some(id) = get_dragged_card(&ev) {
            if part.can_contain(&cards[id]) {
                set_drop_effect(&ev, DropEffect::Copy);
                ev.prevent_default();
            }
        }
    };

    view! {
        <h2>{part.to_string()}</h2>
        <div class="part-size">
            <span class="current">
                {move || {
                    deck.with(|deck| {
                        deck.iter_part(cards, part).map(|(_, count)| count).sum::<usize>()
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
                if let Some(id) = get_dragged_card(&ev) {
                    if part.can_contain(&cards[id]) {
                        deck.update(|deck| {
                            deck.increment(id, part.into(), 1);
                        });
                    }
                }
            }
        >

            <For
                each=move || { deck.with(|deck| deck.iter_part(cards, part).collect::<Vec<_>>()) }
                key=|el: &(Id, usize)| *el
                children=move |(id, count)| {
                    let delete = delete.clone();
                    view! { <CardView id=id count=count on_delete=delete/> }
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
            <PartView part=DeckPart::Main/>
            <PartView part=DeckPart::Extra/>
            <PartView part=DeckPart::Side/>
        </div>
    }
}
