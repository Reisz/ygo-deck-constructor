use bincode::Options;
use data::card::{CardData, Id};
use gloo_net::http::Request;
use leptos::{
    component, create_local_resource, create_node_ref, html, mount_to_body, prelude::*,
    provide_context, use_context, view, For, ForProps, IntoView, Scope, Suspense, SuspenseProps,
};
use lzma_rs::xz_decompress;

async fn load_cards(_: ()) -> &'static CardData {
    let request = Request::get("/cards.bin.xz");
    let response = request.send().await.unwrap();
    let bytes = response.binary().await.unwrap();

    let mut decompressed = Vec::new();
    xz_decompress(&mut bytes.as_slice(), &mut decompressed).unwrap();
    Box::leak(Box::new(
        data::bincode_options().deserialize(&decompressed).unwrap(),
    ))
}

#[component]
fn Card(cx: Scope, id: Id) -> impl IntoView {
    let card = &use_context::<&'static CardData>(cx).unwrap()[&id];

    view! { cx,
        <div
            class="card"
            draggable="true"
            on:dragstart=move |ev| {
                ev.data_transfer().unwrap().set_data("text/plain", &id.get().to_string()).unwrap();
            }
        >
            <div class="name">{&card.name}</div>
        </div>
    }
}

#[component]
fn CardSearch(cx: Scope) -> impl IntoView {
    let cards = use_context::<&'static CardData>(cx).unwrap();

    let (filter, set_filter) = create_signal(cx, String::new());

    let card_iter = move || {
        let filter = filter();
        cards
            .iter()
            .filter(move |(_, card)| {
                if filter.is_empty() {
                    return true;
                }

                card.name.contains(&filter)
            })
            .take(50) // TODO: implement paging
    };

    let input_ref = create_node_ref::<html::Input>(cx);
    view! { cx,
        <div class="card-search">
            <input
                type="text"
                node_ref=input_ref
                on:input=move |_| {
                    let input = input_ref.get().unwrap();
                    set_filter(input.value());
                }
            />
            <div class="card-list">
                <For
                    each=card_iter
                    key=|(id, _)| *id
                    view=move |cx, (id, _)| {
                        view! { cx, <Card id=*id/> }
                    }
                />
            </div>
        </div>
    }
}

#[derive(Debug, Clone, Copy)]
struct DrawerData {
    id: usize,
    name: RwSignal<String>,
    content: RwSignal<Vec<Id>>,
}

#[component]
fn Drawer(cx: Scope, data: DrawerData) -> impl IntoView {
    let close = move || {
        use_context::<WriteSignal<Vec<DrawerData>>>(cx)
            .unwrap()
            .update(|drawers| drawers.retain(|drawer| drawer.id != data.id));
    };

    let iter = move || {
        data.content
            .get()
            .iter()
            .copied()
            .enumerate()
            .collect::<Vec<_>>()
    };

    // TODO: propagate input updates back to name signal
    view! { cx,
        <div class="drawer">
            <input type="text" value=data.name/>
            <button on:click=move |_| close()>"X"</button>
            <div
                class="card-list"
                on:dragenter=move |ev| {
                    ev.prevent_default();
                }
                on:dragover=move |ev| {
                    ev.prevent_default();
                }
                on:drop=move |ev| {
                    let id = Id::new(
                        ev.data_transfer().unwrap().get_data("text/plain").unwrap().parse().unwrap(),
                    );
                    data.content.update(|content| content.push(id));
                }
            >
                <For
                    each=iter
                    key=|(idx, _)| *idx
                    view=move |cx, (_, id)| {
                        view! { cx, <Card id=id/> }
                    }
                />
            </div>
        </div>
    }
}

#[component]
fn Drawers(cx: Scope) -> impl IntoView {
    let (next_drawer_id, set_next_drawer_id) = create_signal(cx, 0);
    let (drawers, set_drawers) = create_signal(cx, Vec::new());

    provide_context(cx, set_drawers);

    let new_drawer = move || {
        set_drawers.update(|drawers| {
            drawers.push(DrawerData {
                id: next_drawer_id(),
                name: create_rw_signal(cx, "New Drawer".to_owned()),
                content: create_rw_signal(cx, Vec::new()),
            });
        });
        set_next_drawer_id.update(|id| *id += 1);
    };

    view! { cx,
        <div class="drawers">
            <For
                each=drawers
                key=|data| data.id
                view=move |cx, data| {
                    view! { cx, <Drawer data=data/> }
                }
            />
            <button on:click=move |_| new_drawer()>"+"</button>
        </div>
    }
}

#[component]
fn DeckPart(cx: Scope, name: &'static str) -> impl IntoView {
    let (content, set_content) = create_signal(cx, Vec::new());
    let iter = move || content().iter().copied().enumerate().collect::<Vec<_>>();

    view! { cx,
        <h2>{name}</h2>
        <div
            class="card-list"
            on:dragenter=move |ev| {
                ev.prevent_default();
            }
            on:dragover=move |ev| {
                ev.prevent_default();
            }
            on:drop=move |ev| {
                let id = Id::new(
                    ev.data_transfer().unwrap().get_data("text/plain").unwrap().parse().unwrap(),
                );
                set_content.update(|content| content.push(id));
            }
        >
            <For
                each=iter
                key=|(idx, _)| *idx
                view=move |cx, (_, id)| {
                    view! { cx, <Card id=id/> }
                }
            />
        </div>
    }
}

#[component]
fn Deck(cx: Scope) -> impl IntoView {
    view! { cx,
        <div class="deck-view">
            <DeckPart name="Main"/>
            <DeckPart name="Side"/>
            <DeckPart name="Extra"/>
        </div>
    }
}

#[component]
fn DeckBuilder(cx: Scope, cards: &'static CardData) -> impl IntoView {
    provide_context(cx, cards);

    view! { cx,
        <div class="deck-builder">
            <CardSearch/>
            <Drawers/>
            <Deck/>
        </div>
    }
}

#[component]
fn App(cx: Scope) -> impl IntoView {
    let cards = create_local_resource(cx, || (), load_cards);

    view! { cx,
        <Suspense fallback=move || "Loading...">
            {move || {
                cards
                    .read(cx)
                    .map(|cards| {
                        view! { cx, <DeckBuilder cards=cards/> }
                    })
            }}
        </Suspense>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|cx| view! { cx, <App/> });
}
