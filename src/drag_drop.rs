use common::card::{Card, Id};
use wasm_bindgen::intern;
use web_sys::{DataTransfer, DragEvent};

const CARD_ID_TYPE: &str = "card_id";

fn data_transfer(ev: &DragEvent) -> DataTransfer {
    ev.data_transfer().expect("data transfer not available")
}

fn set_data(transfer: &DataTransfer, format: &str, data: &str) {
    transfer
        .set_data(intern(format), data)
        .expect("failed setting drag data");
}

pub fn start_drag(ev: &DragEvent, id: Id, card: &Card) {
    let transfer = data_transfer(ev);
    set_data(&transfer, CARD_ID_TYPE, &id.get().to_string());
    set_data(
        &transfer,
        "text/uri-list",
        &format!("https://yugipedia.com/wiki/{}", &card.name),
    );
    set_data(&transfer, "text/plain", &card.name);
}

#[must_use]
pub fn get_dragged_card(ev: &DragEvent) -> Option<Id> {
    let data = data_transfer(ev);
    let data = data
        .get_data(CARD_ID_TYPE)
        .expect("failed getting card data");

    if data.is_empty() {
        return None;
    }

    Some(Id::new(data.parse().expect("failed parsing card data")))
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
