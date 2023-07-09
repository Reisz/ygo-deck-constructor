mod card_view;
mod ydk;

use std::{cmp::Ordering, fmt::Display, rc::Rc};

use bincode::Options;
use card_view::CardView;
use common::card::{Card, CardData, CardType, Id, MonsterStats, MonsterType};
use gloo_net::http::Request;
use leptos::{
    component, create_local_resource, create_node_ref, html, mount_to_body, prelude::*,
    provide_context, use_context, view, For, IntoView, Scope, Suspense,
};
use lzma_rs::xz_decompress;
use ygo_deck_constructor::drag_drop::{get_dragged_card, set_drop_effect, DropEffect};

async fn load_cards(_: ()) -> &'static CardData {
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
                        view! { cx, <CardView id=*id/> }
                    }
                />
            </div>
        </div>
    }
}

fn deck_order(data: &'static CardData, lhs: Id, rhs: Id) -> Ordering {
    let lhs = &data[&lhs];
    let rhs = &data[&rhs];

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
fn Drawer(cx: Scope, data: DrawerData) -> impl IntoView {
    let close = move || {
        use_context::<WriteSignal<Vec<DrawerData>>>(cx)
            .unwrap()
            .update(|drawers| drawers.retain(|drawer| drawer.id != data.id));
    };

    let push = move |id| {
        let cards = use_context::<&'static CardData>(cx).unwrap();
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
    view! { cx,
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
                    view=move |cx, id| {
                        let delete = delete.clone();
                        view! { cx, <CardView id=id on_delete=delete/> }
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

#[derive(Debug, Clone, Copy)]
enum DeckPartType {
    Main,
    Extra,
    Side,
}

impl DeckPartType {
    fn can_contain(self, card: &Card) -> bool {
        let is_extra = matches!(
            card.card_type,
            CardType::Monster {
                stats: MonsterStats::Normal {
                    monster_type: Some(
                        MonsterType::Fusion | MonsterType::Synchro | MonsterType::Xyz
                    ),
                    ..
                } | MonsterStats::Link { .. },
                ..
            }
        );

        match self {
            Self::Main => !is_extra,
            Self::Extra => is_extra,
            Self::Side => true,
        }
    }

    fn max(self) -> u8 {
        match self {
            Self::Main => 60,
            Self::Extra | Self::Side => 15,
        }
    }
}

impl Display for DeckPartType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Main => "Main",
            Self::Extra => "Extra",
            Self::Side => "Side",
        };
        write!(f, "{name}")
    }
}

#[component]
fn DeckPart(cx: Scope, part_type: DeckPartType) -> impl IntoView {
    let (next_idx, set_next_idx) = create_signal(cx, 0usize);
    let (content, set_content) = create_signal(cx, Vec::new());

    let cards = use_context::<&'static CardData>(cx).unwrap();
    let push = move |id| {
        set_content.update(|content| {
            let (Ok(pos) | Err(pos)) =
                content.binary_search_by(|(_, probe)| deck_order(cards, *probe, id));
            content.insert(pos, (next_idx(), id));
            set_next_idx.update(|idx| *idx += 1);
        });
    };

    let delete = move |delete_id| {
        set_content.update(|content| {
            let mut has_deleted = false;
            content.retain(|(_, id)| {
                if *id == delete_id && !has_deleted {
                    has_deleted = true;
                    false
                } else {
                    true
                }
            });
        });
    };
    let delete = Rc::new(delete);

    let drag_over = move |ev| {
        if let Some(id) = get_dragged_card(&ev) {
            if part_type.can_contain(&cards[&id]) {
                set_drop_effect(&ev, DropEffect::Copy);
                ev.prevent_default();
            }
        }
    };

    view! { cx,
        <h2>{part_type.to_string()}</h2>
        <div class="part-size">
            <span class="current">{move || content().len()}</span>
            <span class="divider">" / "</span>
            <span class="max">{part_type.max()}</span>
        </div>
        <div
            class="card-list"
            on:dragenter=drag_over
            on:dragover=drag_over
            on:drop=move |ev| {
                if let Some(id) = get_dragged_card(&ev) {
                    if part_type.can_contain(&cards[&id]) {
                        push(id);
                    }
                }
            }
        >
            <For
                each=content
                key=|(idx, _)| *idx
                view=move |cx, (_, id)| {
                    let delete = delete.clone();
                    view! { cx, <CardView id=id on_delete=delete/> }
                }
            />
        </div>
    }
}

#[component]
fn Deck(cx: Scope) -> impl IntoView {
    view! { cx,
        <div class="deck-view">
            <DeckPart part_type=DeckPartType::Main/>
            <DeckPart part_type=DeckPartType::Extra/>
            <DeckPart part_type=DeckPartType::Side/>
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
