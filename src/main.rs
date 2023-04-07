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
    view! {cx,
        <div class="card">
            <div class="name">{&card.name}</div>
            <div class="usage">
                <button>"-"</button>
                <span class="current">"0"</span>
                <span>"/"</span>
                <span class="limit">{card.limit.count()}</span>
                <button>"+"</button>
            </div>
        </div>
    }
}

#[component]
fn CardList(cx: Scope, cards: &'static CardData) -> impl IntoView {
    view! {
        cx,
        <For
            each = move || cards
            key = |(card_id, _)| *card_id
            view = move |cx, (_, card)| view! {cx, <Card card = card />}
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
                cards.read(cx).map(|cards| view!{cx, <CardList cards = cards />})
            }}
        </Suspense>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|cx| view! { cx, <App /> });
}
