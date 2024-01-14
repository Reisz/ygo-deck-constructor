use std::error::Error;

use common::card_data::CardData;
use gloo_file::{futures::read_as_text, Blob, File};
use leptos::{
    component, expect_context, html, spawn_local, view, IntoView, NodeRef, RwSignal, SignalSet,
    SignalWith,
};
use web_sys::Url;

use crate::{deck::Deck, error_handling::JsException, print_error, ydk};

async fn do_import(file: File, cards: &'static CardData) -> Result<Deck, Box<dyn Error>> {
    leptos::logging::log!("do import");
    Ok(ydk::load(&read_as_text(&file.into()).await?, cards)?)
}

fn do_export(deck: &Deck, cards: &'static CardData) -> Result<(), Box<dyn Error>> {
    let mut buffer = Vec::new();
    ydk::save(deck, cards, &mut buffer)?;

    let blob = Blob::new_with_options(buffer.as_slice(), Some("text/ydk"));
    let url = Url::create_object_url_with_blob(blob.as_ref()).map_err(JsException::from)?;

    view! { <a href=&url download="deck.ydk"></a> }.click();
    Url::revoke_object_url(&url).map_err(JsException::from)?;

    Ok(())
}

#[component]
#[must_use]
pub fn Menu() -> impl IntoView {
    let cards = expect_context::<&'static CardData>();
    let deck = expect_context::<RwSignal<Deck>>();

    let input_ref = NodeRef::<html::Input>::new();
    let import = move |_| {
        let input = input_ref.get().unwrap();
        let files = input.files().unwrap(/* should only be null if type!=file */);
        if let Some(file) = files.get(0) {
            spawn_local(async move {
                let name = file.name();
                match do_import(file.into(), cards).await {
                    Ok(new_deck) => deck.set(new_deck),
                    Err(err) => print_error!("Error while importing \"{name}\":\n\n{err}"),
                }
            });
        }
    };

    let export = move |_| match deck.with(|deck| do_export(deck, cards)) {
        Ok(()) => {}
        Err(err) => print_error!("Error while exporting:\n\n{err}"),
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
