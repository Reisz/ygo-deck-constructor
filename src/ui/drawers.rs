use std::rc::Rc;

use common::card_data::{CardData, Id};
use leptos::{
    component, create_rw_signal, create_signal, expect_context, view, For, IntoView, RwSignal,
    SignalGet, SignalUpdate, SignalWith, WriteSignal,
};

use crate::{
    deck_order::deck_order,
    ui::{
        card_view::CardView,
        drag_drop::{get_drag_info, get_dropped_card, set_drop_effect, DragInfo, DropEffect},
    },
};

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

    let cards = expect_context::<CardData>();
    let push = move |id| {
        data.content.update(|content| {
            if let Err(pos) =
                content.binary_search_by(|probe| deck_order(&cards[*probe], &cards[id]))
            {
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
        if !matches!(get_drag_info(&ev), DragInfo::NotCard) {
            set_drop_effect(&ev, DropEffect::Copy);
            ev.prevent_default();
        }
    };

    // TODO: propagate input updates back to name signal
    view! {
        <div class="drawer">
            <input type="text" value=data.name />
            <button on:click=move |_| close()>"X"</button>
            <div
                class="card-list"
                on:dragenter=drag_over
                on:dragover=drag_over
                on:drop=move |ev| {
                    let id = get_dropped_card(&ev, &cards);
                    if !data.content.with(|content| content.contains(&id)) {
                        push(id);
                    }
                }
            >

                <For
                    each=move || data.content.get()
                    key=|id| *id
                    children=move |id| {
                        let delete = delete.clone();
                        view! { <CardView id=id on_delete=delete /> }
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
                id: next_drawer_id.get(),
                name: create_rw_signal("New Drawer".to_owned()),
                content: create_rw_signal(Vec::new()),
            });
        });
        set_next_drawer_id.update(|id| *id += 1);
    };

    view! {
        <div class="drawers">
            <For
                each=move || drawers.get()
                key=|data| data.id
                children=move |data| {
                    view! { <Drawer data=data set_drawers=set_drawers /> }
                }
            />

            <button on:click=move |_| new_drawer()>"+"</button>
        </div>
    }
}
