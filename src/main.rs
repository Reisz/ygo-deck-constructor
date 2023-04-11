use bincode::Options;
use data::card::{Card, CardData, Id};
use gloo_net::http::Request;
use leptos::{
    component, create_local_resource, create_node_ref, create_signal, html::Input, log,
    mount_to_body, view, For, ForProps, IntoView, Scope, SignalUpdate, Suspense, SuspenseProps,
};
use lzma_rs::xz_decompress;

async fn load_cards(_: ()) -> &'static CardData {
    let request = Request::get("/cards.bin.xz");
    let response = request.send().await.unwrap();
    let bytes = response.binary().await.unwrap();

    let mut decompressed = Vec::new();
    xz_decompress(&mut bytes.as_slice(), &mut decompressed).unwrap();
    Box::leak(Box::new(
        data::bincode_options().deserialize(&decompressed).unwrap(),
    ))
}

#[component]
fn Card(cx: Scope, card: &'static Card) -> impl IntoView {
    view! {cx,
        <div
            class="card"
            draggable="true"
            on:dragstart=|_| {}
        >
            <div class="name">{&card.name}</div>
        </div>
    }
}

#[component]
fn CardSearch(cx: Scope, cards: &'static CardData) -> impl IntoView {
    let (filter, set_filter) = create_signal(cx, String::new());

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

    let input_ref = create_node_ref::<Input>(cx);
    view! {
        cx,
        <div class="card-search">
            <input
                type = "text"
                node_ref = input_ref
                on:input = move |_| {
                    let input = input_ref.get().unwrap();
                    set_filter(input.value());
                }
            />
            <div class="card-list">
                <For
                    each = card_iter
                    key = |(card_id, _)| *card_id
                    view = move |cx, (_, card)| view! {cx, <Card card = card />}
                />
            </div>
        </div>
    }
}

#[component]
fn DeckPart(cx: Scope, cards: &'static CardData, name: &'static str) -> impl IntoView {
    let (content, set_content) = create_signal(cx, Vec::new());

    view! { cx,
        <h2>{name}</h2>
        <div
            class = "card-list"
            on:dragenter = move |ev| {ev.prevent_default();}
            on:dragover = move |ev| {ev.prevent_default();}
            on:drop = move |_| {
                set_content.update(|content| {content.push(Id::new(6983839)); log!("{:?}", content);});
            }
        >
            <For
                each = {move || content().iter().copied().enumerate().collect::<Vec<_>>()}
                key = |(idx, _)| *idx
                view = move |cx, (_, card_id)| view! {cx, <Card card = &cards[&card_id] />}
            />
        </div>
    }
}

#[component]
fn Deck(cx: Scope, cards: &'static CardData) -> impl IntoView {
    view! {
        cx,
        <div class="deck-view">
            <DeckPart cards=cards name="Main" />
            <DeckPart cards=cards name="Side" />
            <DeckPart cards=cards name="Extra" />
        </div>
    }
}

#[component]
fn DeckBuilder(cx: Scope, cards: &'static CardData) -> impl IntoView {
    view! {cx,
        <div class="deck-builder">
            <CardSearch cards = cards />
            <Deck cards = cards />
        </div>
    }
}

#[component]
fn App(cx: Scope) -> impl IntoView {
    let cards = create_local_resource(cx, || (), load_cards);

    view! {
        cx,
        <Suspense fallback = move || "Loading...">
            {move || {
                cards.read(cx).map(|cards| view!{cx, <DeckBuilder cards = cards />})
            }}
        </Suspense>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|cx| view! { cx, <App /> });
}
