use common::{
    card::Card,
    card_data::{CardData, Id},
};
use wasm_bindgen::intern;
use web_sys::{js_sys::JsString, DataTransfer, DragEvent};

const CARD_PASSWORD_TYPE: &str = "card_password";
const CARD_IS_EXTRA: &str = "card_is_extra";

fn data_transfer(ev: &DragEvent) -> DataTransfer {
    ev.data_transfer().expect("data transfer not available")
}

fn set_data(transfer: &DataTransfer, format: &str, data: &str) {
    transfer
        .set_data(intern(format), data)
        .expect("failed setting drag data");
}

pub fn start_drag(ev: &DragEvent, card: &Card) {
    let transfer = data_transfer(ev);

    set_data(&transfer, CARD_PASSWORD_TYPE, &card.password.to_string());
    if card.card_type.is_extra_deck_monster() {
        // Marker for dragover, so content does not matter
        set_data(&transfer, CARD_IS_EXTRA, "");
    }

    set_data(
        &transfer,
        "text/uri-list",
        &format!("https://yugipedia.com/wiki/{}", card.name),
    );
    set_data(&transfer, "text/plain", card.name);
}

pub enum DragInfo {
    NotCard,
    MainCard,
    ExtraCard,
}

/// Get all the info available during `dragenter` and `dragover`.
#[must_use]
pub fn get_drag_info(ev: &DragEvent) -> DragInfo {
    // NOTE: some browsers clear out the data during `dragenter` and `dragover`, so we can only
    // rely on the `types` property. See https://stackoverflow.com/a/28487486

    let types = data_transfer(ev).types();

    if !types.includes(&JsString::from(intern(CARD_PASSWORD_TYPE)), 0) {
        return DragInfo::NotCard;
    }

    if types.includes(&JsString::from(intern(CARD_IS_EXTRA)), 0) {
        DragInfo::ExtraCard
    } else {
        DragInfo::MainCard
    }
}

/// Get the password of the dropped card.
///
/// Only available in the `drop` event.
#[must_use]
pub fn get_dropped_card(ev: &DragEvent, cards: &CardData) -> Id {
    let data = data_transfer(ev);
    let data = data
        .get_data(CARD_PASSWORD_TYPE)
        .expect("failed getting card password");
    let password = data.parse().expect("failed parsing card password");
    cards
        .id_for_password(password)
        .expect("unknown card password")
}

#[derive(Debug, Copy, Clone)]
pub enum DropEffect {
    None,
    Copy,
    Move,
    Link,
}

pub fn set_drop_effect(ev: &DragEvent, drop_effect: DropEffect) {
    let effect = match drop_effect {
        DropEffect::None => "none",
        DropEffect::Copy => "copy",
        DropEffect::Move => "move",
        DropEffect::Link => "link",
    };
    data_transfer(ev).set_drop_effect(intern(effect));
}
