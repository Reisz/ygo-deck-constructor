use bincode::Options;
use data::card::{Card, CardData};
use gloo_net::http::Request;
use leptos::{
    component, create_local_resource, mount_to_body, view, For, ForProps, IntoView, Scope,
    Suspense, SuspenseProps,
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
    view! {
        cx,
        <h3>{&card.name}</h3>
        <For
            each = move || card.description.lines().enumerate()
            key = |(i, _)| *i
            view = move |cx, (_, line)| {
                view! {
                    cx,
                    <p>{line}</p>
                }
            }
        />
    }
}

#[component]
fn CardList(cx: Scope, cards: &'static CardData) -> impl IntoView {
    view! {
        cx,
        <For
            each = move || cards
            key = |(card_id, _)| *card_id
            view = move |cx, (_, card)| {
                view! {
                    cx,
                    <Card card = card />
                }
            }
        />
    }
}

#[component]
fn App(cx: Scope) -> impl IntoView {
    let cards = create_local_resource(cx, || (), load_cards);

    view! {
        cx,
        <Suspense fallback = move || "Loading...">
            {move || {
                cards.read(cx).map(|cards| view!{cx,
                    <h1>{ "Card List" }</h1>
                    <CardList cards = cards />
                })
            }}
        </Suspense>

    }
}

fn main() {
    mount_to_body(|cx| view! { cx, <App /> });
}
