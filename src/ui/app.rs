use bincode::Options;
use common::{self, card::CardData};
use gloo_net::http::Request;
use leptos::{
    component, create_local_resource, provide_context, view, IntoView, RwSignal, Suspense,
};
use lzma_rs::xz_decompress;

use crate::{
    deck::Deck,
    ui::{card_search::CardSearch, deck_view::DeckView, drawers::Drawers, tools::Tools},
};

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
#[must_use]
pub fn App() -> impl IntoView {
    let cards = create_local_resource(|| 1, |_| load_cards());

    let fallback = move || "Loading...";
    let app = move || {
        cards.map(|cards| {
            provide_context::<&'static CardData>(*cards);
            provide_context(RwSignal::new(Deck::new()));

            view! {
                <div class="deck-builder">
                    <CardSearch/>
                    <Drawers/>
                    <DeckView/>
                    <Tools/>
                </div>
            }
        })
    };

    view! { <Suspense fallback=fallback>{app}</Suspense> }
}
