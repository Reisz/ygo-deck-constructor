use std::error::Error;

use common::{card_data::CardData, ydk};
use gloo_file::{futures::read_as_text, Blob, File};
use leptos::{
    component, create_effect, expect_context, html, logging, provide_context, spawn_local, view,
    IntoView, NodeRef, RwSignal, SignalSet, SignalUpdate, SignalWith,
};
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{KeyboardEvent, Url};

use crate::{deck::Deck, error_handling::JsException, print_error, text_encoding::TextEncoding};

async fn do_import(file: File, cards: &CardData) -> Result<Deck, Box<dyn Error>> {
    Ok(Deck::new(ydk::load(
        &read_as_text(&file.into()).await?,
        cards,
    )?))
}

fn do_export(deck: &Deck, cards: &CardData) -> Result<(), Box<dyn Error>> {
    let mut buffer = Vec::new();
    ydk::save(deck, cards, &mut buffer)?;

    let blob = Blob::new_with_options(buffer.as_slice(), Some("text/ydk"));
    let url = Url::create_object_url_with_blob(blob.as_ref()).map_err(JsException::from)?;

    view! { <a href=&url download="deck.ydk"></a> }.click();
    Url::revoke_object_url(&url).map_err(JsException::from)?;

    Ok(())
}

fn install_undo_redo_shortcuts(deck: RwSignal<Deck>) {
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
}
/// Install the main deck instance as leptos context
pub fn install_as_context() {
    const KEY: &str = "deck";
    let storage = leptos::window().local_storage().ok().flatten();
    let deck = storage
        .as_ref()
        .and_then(|storage| storage.get_item(KEY).ok().flatten())
        .as_deref()
        .and_then(Deck::decode)
        .unwrap_or_default();
    let deck = RwSignal::new(deck);

    if let Some(storage) = storage {
        create_effect(move |_| {
            let text = deck.with(TextEncoding::encode_string);
            if storage.set_item(KEY, &text).is_err() {
                logging::error!("Saving deck failed");
            }
        });
    }

    install_undo_redo_shortcuts(deck);
    provide_context(deck);
}

#[component]
#[must_use]
pub fn Menu() -> impl IntoView {
    let cards = expect_context::<CardData>();
    let deck = expect_context::<RwSignal<Deck>>();

    let input_ref = NodeRef::<html::Input>::new();
    let import = move |_| {
        let input = input_ref.get().unwrap();
        let files = input.files().unwrap(/* should only be null if type!=file */);
        if let Some(file) = files.get(0) {
            spawn_local(async move {
                let name = file.name();
                match do_import(file.into(), &cards).await {
                    Ok(new_deck) => deck.set(new_deck),
                    Err(err) => print_error!("Error while importing \"{name}\":\n\n{err}"),
                }
            });
        }
    };

    let export = move |_| match deck.with(|deck| do_export(deck, &cards)) {
        Ok(()) => {}
        Err(err) => print_error!("Error while exporting:\n\n{err}"),
    };

    view! {
        <div class="menu">
            <button on:click=move |_| deck.set(Deck::default())>"New"</button>
            <button on:click:undelegated=move |_| {
                input_ref.get().unwrap().click();
            }>"Import..."</button>
            <button on:click=export>"Export..."</button>
            <input type="file" accept=".ydk" ref=input_ref on:change=import style="display: none" />
        </div>
    }
}
