use std::{cmp::Ordering, rc::Rc};

use common::{card::Id, card_data::CardData};
use leptos::{
    component, create_rw_signal, create_signal, expect_context, view, For, IntoView, RwSignal,
    SignalGet, SignalUpdate, WriteSignal,
};

use crate::ui::{
    card_view::CardView,
    drag_drop::{get_dragged_card, set_drop_effect, DropEffect},
};

fn deck_order(data: &CardData, lhs: Id, rhs: Id) -> Ordering {
    let lhs = &data[lhs];
    let rhs = &data[rhs];

    // TODO: order by card type first
    lhs.name.cmp(&rhs.name)
}

#[derive(Debug, Clone, Copy)]
struct DrawerData {
    id: usize,
    name: RwSignal<String>,
    content: RwSignal<Vec<Id>>,
}

#[component]
fn Drawer(data: DrawerData, set_drawers: WriteSignal<Vec<DrawerData>>) -> impl IntoView {
    let close = move || {
        set_drawers.update(|drawers| drawers.retain(|drawer| drawer.id != data.id));
    };

    let cards = expect_context::<&'static CardData>();
    let push = move |id| {
        data.content.update(|content| {
            if let Err(pos) = content.binary_search_by(|probe| deck_order(cards, *probe, id)) {
                content.insert(pos, id);
            }
        });
    };

    let delete = move |delete_id| {
        data.content
            .update(|content| content.retain(|id| *id != delete_id));
    };
    let delete: Rc<dyn Fn(Id)> = Rc::new(delete);

    let drag_over = move |ev| {
        if let Some(id) = get_dragged_card(&ev) {
            if !data.content.get().contains(&id) {
                set_drop_effect(&ev, DropEffect::Copy);
                ev.prevent_default();
            }
        }
    };

    // TODO: propagate input updates back to name signal
    view! {
        <div class="drawer">
            <input type="text" value=data.name/>
            <button on:click=move |_| close()>"X"</button>
            <div
                class="card-list"
                on:dragenter=drag_over
                on:dragover=drag_over
                on:drop=move |ev| {
                    if let Some(id) = get_dragged_card(&ev) {
                        push(id);
                    }
                }
            >

                <For
                    each=data.content
                    key=|id| *id
                    children=move |id| {
                        let delete = delete.clone();
                        view! { <CardView id=id on_delete=delete/> }
                    }
                />

            </div>
        </div>
    }
}

#[component]
#[must_use]
pub fn Drawers() -> impl IntoView {
    let (next_drawer_id, set_next_drawer_id) = create_signal(0);
    let (drawers, set_drawers) = create_signal(Vec::new());

    let new_drawer = move || {
        set_drawers.update(|drawers| {
            drawers.push(DrawerData {
                id: next_drawer_id(),
                name: create_rw_signal("New Drawer".to_owned()),
                content: create_rw_signal(Vec::new()),
            });
        });
        set_next_drawer_id.update(|id| *id += 1);
    };

    view! {
        <div class="drawers">
            <For
                each=drawers
                key=|data| data.id
                children=move |data| {
                    view! { <Drawer data=data set_drawers=set_drawers/> }
                }
            />

            <button on:click=move |_| new_drawer()>"+"</button>
        </div>
    }
}
