use common::card::CardData;
use gloo_file::{futures::read_as_text, Blob};
use leptos::{
    component, expect_context, html, spawn_local, view, IntoView, NodeRef, RwSignal, SignalSet,
    SignalWith,
};
use web_sys::Url;

use crate::{deck::Deck, ydk};

#[component]
#[must_use]
pub fn Menu() -> impl IntoView {
    let cards = expect_context::<&'static CardData>();
    let deck = expect_context::<RwSignal<Deck>>();

    let input_ref = NodeRef::<html::Input>::new();
    let import = move |_| {
        let input = input_ref.get().unwrap();
        if let Some(file) = input.files().and_then(|files| files.get(0)) {
            spawn_local(async move {
                deck.set(ydk::load(&read_as_text(&file.into()).await.unwrap()).unwrap());
            });
        }
    };

    let export = move |_| {
        let mut buffer = Vec::new();
        deck.with(|deck| ydk::save(deck, cards, &mut buffer).unwrap());

        let blob = Blob::new_with_options(buffer.as_slice(), Some("text/ydk"));
        let url = Url::create_object_url_with_blob(blob.as_ref()).unwrap();

        view! { <a href=&url download="deck.ydk"></a> }.click();
        Url::revoke_object_url(&url).unwrap();
    };

    view! {
        <div class="menu">
            <button on:click=move |_| deck.set(Deck::default())>"New"</button>
            <button on:click=move |_| input_ref.get().unwrap().click()>"Import..."</button>
            <button on:click=export>"Export..."</button>
            <input type="file" accept=".ydk" ref=input_ref on:change=import style="display: none"/>
        </div>
    }
}
