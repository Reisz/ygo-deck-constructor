mod card_view;
mod deck;
mod ydk;

use std::{cmp::Ordering, ops::Deref, rc::Rc};

use bincode::Options;
use card_view::CardView;
use common::card::{CardData, Id};
use gloo_net::http::Request;
use leptos::{
    component, create_local_resource, create_node_ref, expect_context, html, mount_to_body,
    prelude::*, provide_context, view, For, IntoView, Suspense,
};
use lzma_rs::xz_decompress;
use ygo_deck_constructor::drag_drop::{get_dragged_card, set_drop_effect, DropEffect};

use crate::deck::{Deck, DeckPart, PartType};

async fn load_cards() -> &'static CardData {
    let request = Request::get("cards.bin.xz");
    let response = request.send().await.unwrap();
    let bytes = response.binary().await.unwrap();

    let mut decompressed = Vec::new();
    xz_decompress(&mut bytes.as_slice(), &mut decompressed).unwrap();
    Box::leak(Box::new(
        common::bincode_options()
            .deserialize(&decompressed)
            .unwrap(),
    ))
}

#[component]
fn CardSearch() -> impl IntoView {
    let cards = expect_context::<&'static CardData>();

    let (filter, set_filter) = create_signal(String::new());

    let card_iter = move || {
        let filter = filter();
        cards
            .iter()
            .filter(move |(_, card)| {
                if filter.is_empty() {
                    return true;
                }

                card.name.contains(&filter)
            })
            .take(50) // TODO: implement paging
    };

    let input_ref = create_node_ref::<html::Input>();
    view! {
        <div class="card-search">
            <input
                type="text"
                node_ref=input_ref
                on:input=move |_| {
                    let input = input_ref.get().unwrap();
                    set_filter(input.value());
                }
            />

            <div class="card-list">
                <For
                    each=card_iter
                    key=|(id, _)| *id
                    view=move |(id, _)| {
                        view! { <CardView id=*id/> }
                    }
                />

            </div>
        </div>
    }
}

fn deck_order(data: &CardData, lhs: Id, rhs: Id) -> Ordering {
    let lhs = &data[&lhs];
    let rhs = &data[&rhs];

    // TODO: order by card type first
    lhs.name.cmp(&rhs.name)
}

#[derive(Debug, Clone, Copy)]
struct DrawerData {
    id: usize,
    name: RwSignal<String>,
    content: RwSignal<Vec<Id>>,
}

#[component]
fn Drawer(data: DrawerData, set_drawers: WriteSignal<Vec<DrawerData>>) -> impl IntoView {
    let close = move || {
        set_drawers.update(|drawers| drawers.retain(|drawer| drawer.id != data.id));
    };

    let cards = expect_context::<&'static CardData>();
    let push = move |id| {
        data.content.update(|content| {
            if let Err(pos) = content.binary_search_by(|probe| deck_order(cards, *probe, id)) {
                content.insert(pos, id);
            }
        });
    };

    let delete = move |delete_id| {
        data.content
            .update(|content| content.retain(|id| *id != delete_id));
    };
    let delete: Rc<dyn Fn(Id)> = Rc::new(delete);

    let drag_over = move |ev| {
        if let Some(id) = get_dragged_card(&ev) {
            if !data.content.get().contains(&id) {
                set_drop_effect(&ev, DropEffect::Copy);
                ev.prevent_default();
            }
        }
    };

    // TODO: propagate input updates back to name signal
    view! {
        <div class="drawer">
            <input type="text" value=data.name/>
            <button on:click=move |_| close()>"X"</button>
            <div
                class="card-list"
                on:dragenter=drag_over
                on:dragover=drag_over
                on:drop=move |ev| {
                    if let Some(id) = get_dragged_card(&ev) {
                        push(id);
                    }
                }
            >

                <For
                    each=data.content
                    key=|id| *id
                    view=move |id| {
                        let delete = delete.clone();
                        view! { <CardView id=id on_delete=delete/> }
                    }
                />

            </div>
        </div>
    }
}

#[component]
fn Drawers() -> impl IntoView {
    let (next_drawer_id, set_next_drawer_id) = create_signal(0);
    let (drawers, set_drawers) = create_signal(Vec::new());

    let new_drawer = move || {
        set_drawers.update(|drawers| {
            drawers.push(DrawerData {
                id: next_drawer_id(),
                name: create_rw_signal("New Drawer".to_owned()),
                content: create_rw_signal(Vec::new()),
            });
        });
        set_next_drawer_id.update(|id| *id += 1);
    };

    view! {
        <div class="drawers">
            <For
                each=drawers
                key=|data| data.id
                view=move |data| {
                    view! { <Drawer data=data set_drawers=set_drawers/> }
                }
            />

            <button on:click=move |_| new_drawer()>"+"</button>
        </div>
    }
}

#[component]
fn DeckPart(deck: Deck, part_type: PartType) -> impl IntoView {
    let part = deck[part_type];

    let cards = expect_context::<&'static CardData>();

    let delete = move |delete_id| {
        part.update(|part| part.remove(cards, delete_id));
    };
    let delete = Rc::new(delete);

    let drag_over = move |ev| {
        if let Some(id) = get_dragged_card(&ev) {
            if part_type.can_contain(&cards[&id]) {
                set_drop_effect(&ev, DropEffect::Copy);
                ev.prevent_default();
            }
        }
    };

    view! {
        <h2>{part_type.to_string()}</h2>
        <div class="part-size">
            <span class="current">{move || part.with(DeckPart::len)}</span>
            <span class="divider">" / "</span>
            <span class="max">{part_type.max()}</span>
        </div>
        <div
            class="card-list"
            on:dragenter=drag_over
            on:dragover=drag_over
            on:drop=move |ev| {
                if let Some(id) = get_dragged_card(&ev) {
                    if part_type.can_contain(&cards[&id]) {
                        part.update(|part| part.add(cards, id));
                    }
                }
            }
        >

            <For
                each=move || part.with(|part| part.deref().clone())
                key=|el| *el
                view=move |(id, count)| {
                    let delete = delete.clone();
                    view! { <CardView id=id count=count on_delete=delete/> }
                }
            />

        </div>
    }
}

#[component]
fn DeckView() -> impl IntoView {
    let deck = Deck::new(deck_order);
    view! {
        <div class="deck-view">
            <DeckPart deck=deck part_type=PartType::Main/>
            <DeckPart deck=deck part_type=PartType::Extra/>
            <DeckPart deck=deck part_type=PartType::Side/>
        </div>
    }
}

#[component]
fn DeckBuilder(cards: &'static CardData) -> impl IntoView {
    provide_context(cards);

    view! {
        <div class="deck-builder">
            <CardSearch/>
            <Drawers/>
            <DeckView/>
        </div>
    }
}

#[component]
fn App() -> impl IntoView {
    let cards = create_local_resource(|| 1, |_| load_cards());

    view! {
        <Suspense fallback=move || {
            "Loading..."
        }>
            {move || {
                cards
                    .map(|cards| {
                        view! { <DeckBuilder cards=cards/> }
                    })
            }}

        </Suspense>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> });
}
