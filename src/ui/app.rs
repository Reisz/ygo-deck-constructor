use bincode::Options;
use common::{self, card_data::CardData};
use gloo_net::http::Request;
use leptos::{
    component, create_local_resource, expect_context, provide_context, view, IntoView, RwSignal,
    SignalUpdate, Suspense,
};
use lzma_rs::xz_decompress;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::KeyboardEvent;

use crate::{
    deck::Deck,
    ui::{
        card_search::CardSearch, card_view::CardTooltip, deck_view::DeckView, drawers::Drawers,
        menu::Menu, tools::Tools,
    },
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
            provide_context(RwSignal::new(Deck::default()));

            // TODO find a better place for this
            let deck = expect_context::<RwSignal<Deck>>();
            let keyup = Closure::<dyn Fn(KeyboardEvent)>::new(move |ev: KeyboardEvent| {
                let key = if ev.shift_key() {
                    ev.key().to_uppercase()
                } else {
                    ev.key()
                };

                if ev.ctrl_key() || ev.meta_key() {
                    match key.as_str() {
                        "z" => deck.update(Deck::undo),
                        "y" | "Z" => deck.update(Deck::redo),
                        _ => {}
                    }
                }
            });
            leptos::document().set_onkeyup(Some(keyup.as_ref().unchecked_ref()));
            keyup.forget();

            view! {
                <CardTooltip/>
                <div class="deck-builder">
                    <CardSearch/>
                    <Drawers/>
                    <DeckView/>
                    <div class="extras">
                        <Menu/>
                        <Tools/>
                    </div>
                </div>
            }
        })
    };

    view! { <Suspense fallback=fallback>{app}</Suspense> }
}
