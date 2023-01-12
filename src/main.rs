use std::collections::HashMap;

use bincode::Options;
use data::card::{Card, Id};
use gloo_net::http::Request;
use lzma_rs::xz_decompress;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
struct CardListProps {
    cards: HashMap<Id, Card>,
}

#[function_component(CardList)]
fn card_list(CardListProps { cards }: &CardListProps) -> Html {
    cards
        .iter()
        .map(|(_, card)| {
            html! {
                <>
                    <h3>{&card.name}</h3>
                    {card.desc.lines().map(|l| html!{
                        <p>{l}</p>
                    }).collect::<Html>()}
                </>
            }
        })
        .collect()
}

#[function_component(App)]
fn app() -> Html {
    let cards = use_state(HashMap::new);
    {
        let cards = cards.clone();
        use_effect_with_deps(
            move |_| {
                wasm_bindgen_futures::spawn_local(async move {
                    let request = Request::get("/cards.bin.xz");
                    let response = request.send().await.unwrap();
                    let bytes = response.binary().await.unwrap();

                    let mut decompressed = Vec::new();
                    xz_decompress(&mut bytes.as_slice(), &mut decompressed).unwrap();
                    cards.set(data::bincode_options().deserialize(&decompressed).unwrap());
                });
                || ()
            },
            (),
        );
    }

    html! {
        <>
            <h1>{ "Card List" }</h1>
            <CardList cards={(*cards).clone()} />
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
