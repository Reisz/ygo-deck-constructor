mod card_view;
mod deck;
mod deck_part;
mod ydk;

use std::{cmp::Ordering, rc::Rc};

use bincode::Options;
use card_view::CardView;
use common::card::{Card, CardData, CardLimit, Id};
use deck::PartType;
use gloo_net::http::Request;
use leptos::{
    component, create_local_resource, create_node_ref, expect_context, html, mount_to_body,
    prelude::*, provide_context, view, For, IntoView, Suspense,
};
use lzma_rs::xz_decompress;
use ygo_deck_constructor::drag_drop::{get_dragged_card, set_drop_effect, DropEffect};

use crate::{deck::Deck, deck_part::DeckPart};

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

#[derive(Debug, Default, Clone)]
struct CardFilter {
    name: String,
    text: String,
}

impl CardFilter {
    fn matches(&self, card: &Card) -> bool {
        if !self.name.is_empty() && !card.name.to_ascii_lowercase().contains(&self.name) {
            return false;
        }

        if !self.text.is_empty() && !card.description.to_ascii_lowercase().contains(&self.text) {
            return false;
        }

        true
    }

    fn set_name_filter(&mut self, filter: &str) {
        self.name = filter.to_ascii_lowercase();
    }

    fn set_text_filter(&mut self, filter: &str) {
        self.text = filter.to_ascii_lowercase();
    }
}

#[component]
fn CardSearch() -> impl IntoView {
    let cards = expect_context::<&'static CardData>();

    let (filter, set_filter) = create_signal(CardFilter::default());

    let card_iter = move || {
        cards
            .iter()
            .filter(move |(_, card)| filter.with(|filter| filter.matches(card)))
            .take(50) // TODO: implement paging
    };

    let name_input_ref = create_node_ref::<html::Input>();
    let text_input_ref = create_node_ref::<html::Input>();
    view! {
        <div class="card-search">
            <div class="card-search-params">
                <input
                    type="text"
                    placeholder="Name"
                    node_ref=name_input_ref
                    on:input=move |_| {
                        let input = name_input_ref.get().unwrap();
                        set_filter.update(|filter| filter.set_name_filter(&input.value()));
                    }
                />

                <input
                    type="text"
                    placeholder="Description"
                    node_ref=text_input_ref
                    on:input=move |_| {
                        let input = text_input_ref.get().unwrap();
                        set_filter.update(|filter| filter.set_text_filter(&input.value()));
                    }
                />

            </div>

            <div class="card-list">
                <For
                    each=card_iter
                    key=|(id, _)| *id
                    children=move |(id, _)| {
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
                    children=move |id| {
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
                children=move |data| {
                    view! { <Drawer data=data set_drawers=set_drawers/> }
                }
            />

            <button on:click=move |_| new_drawer()>"+"</button>
        </div>
    }
}

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
            if part.can_contain(&cards[&id]) {
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
                    if part.can_contain(&cards[&id]) {
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
fn DeckView() -> impl IntoView {
    view! {
        <div class="deck-view">
            <PartView part=DeckPart::Main/>
            <PartView part=DeckPart::Extra/>
            <PartView part=DeckPart::Side/>
        </div>
    }
}

#[component]
fn ErrorList() -> impl IntoView {
    let deck = expect_context::<RwSignal<Deck>>();
    let cards = expect_context::<&'static CardData>();

    let errors = move || {
        let mut errors = Vec::new();

        deck.with(|deck| {
            for part in DeckPart::iter() {
                let len = deck
                    .iter_part(cards, part)
                    .map(|(_, count)| count)
                    .sum::<usize>();

                if len < part.min().into() {
                    errors.push(format!(
                        "{part} deck contains less than {} cards ({len})",
                        part.min(),
                    ));
                } else if len > part.max().into() {
                    errors.push(format!(
                        "{part} deck contains more than {} cards ({len})",
                        part.max(),
                    ));
                }
            }

            for entry in deck.entries() {
                let card = &cards[&entry.id()];
                let count = entry.count(PartType::Playing) + entry.count(PartType::Side);

                let limit = card.limit.count();
                let term = match card.limit {
                    CardLimit::Banned => "banned",
                    CardLimit::Limited => "limited",
                    CardLimit::SemiLimited => "semi-limited",
                    CardLimit::Unlimited => "unlimited",
                };

                if count > limit.into() {
                    errors.push(format!(
                        "Card \"{}\" appears {count} times, but is {term} ({limit})",
                        card.name
                    ));
                }
            }
        });

        errors
    };

    view! {
        <h3>{move || { if errors().is_empty() { "No Errors" } else { "Errors" } }}</h3>
        <ul>
            <For
                each=errors
                key=Clone::clone
                children=move |error| {
                    view! { <li>{error}</li> }
                }
            />

        </ul>
    }
}

#[component]
fn Tools() -> impl IntoView {
    view! {
        <div class="tools">
            <ErrorList/>
        </div>
    }
}

#[component]
fn DeckBuilder(cards: &'static CardData) -> impl IntoView {
    provide_context(cards);
    provide_context(RwSignal::new(Deck::new()));

    view! {
        <div class="deck-builder">
            <CardSearch/>
            <Drawers/>
            <DeckView/>
            <Tools/>
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
