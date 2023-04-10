use bincode::Options;
use data::card::{Card, CardData};
use gloo_net::http::Request;
use leptos::{
    component, create_local_resource, create_node_ref, create_signal, html::Input, mount_to_body,
    view, For, ForProps, IntoView, Scope, Suspense, SuspenseProps,
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
    }
}

#[component]
fn Deck(cx: Scope, cards: &'static CardData) -> impl IntoView {
    let (main, _set_main) = create_signal(cx, Vec::new());
    let (extra, _set_extra) = create_signal(cx, Vec::new());
    let (side, _set_side) = create_signal(cx, Vec::new());

    macro_rules! deck_view {
        ($title:expr, $data:expr) => {
            view! { cx,
                <h2>$title</h2>
                <div class="card-list">
                    <For
                        each = $data
                        key = |card_id| *card_id
                        view = move |cx, card_id| view! {cx, <Card card = &cards[card_id] />}
                    />
                </div>
            }
        };
    }

    (
        deck_view!("Main", main),
        deck_view!("Extra", extra),
        deck_view!("Side", side),
    )
}

#[component]
fn DeckBuilder(cx: Scope, cards: &'static CardData) -> impl IntoView {
    view! {cx,
        <CardSearch cards = cards />
        <Deck cards = cards />
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
