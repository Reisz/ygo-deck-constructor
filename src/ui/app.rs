use bincode::Options;
use common::{
    card_data::{CardData, CardDataStorage},
    transfer,
};
use gloo_net::http::Request;
use leptos::{component, create_local_resource, provide_context, view, IntoView, Suspense};
use lzma_rs::xz_decompress;

use crate::ui::{
    card_search::CardSearch, card_view::CardTooltip, deck::Menu, deck_view::DeckView,
    drawers::Drawers, tools::Tools,
};

async fn load_cards() -> CardData {
    let request = Request::get(transfer::DATA_FILENAME);
    let response = request.send().await.unwrap();
    let bytes = response.binary().await.unwrap();

    let mut decompressed = Vec::new();
    xz_decompress(&mut bytes.as_slice(), &mut decompressed).unwrap();
    let cards: CardDataStorage = transfer::bincode_options()
        .deserialize(&decompressed)
        .unwrap();
    cards.into()
}

#[component]
#[must_use]
pub fn App() -> impl IntoView {
    let cards = create_local_resource(|| (), |()| load_cards());

    let fallback = move || "Loading...";
    let app = move || {
        cards.map(|cards| {
            provide_context::<CardData>(*cards);
            crate::ui::deck::install_as_context();

            view! {
                <CardTooltip />
                <div class="deck-builder">
                    <CardSearch />
                    <Drawers />
                    <DeckView />
                    <div class="extras">
                        <Menu />
                        <Tools />
                    </div>
                </div>
            }
        })
    };

    view! { <Suspense fallback=fallback>{app}</Suspense> }
}
