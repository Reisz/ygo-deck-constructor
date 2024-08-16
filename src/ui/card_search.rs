use common::{card::Card, card_data::CardData};
use leptos::{
    component, create_memo, create_node_ref, create_signal, expect_context, html, provide_context,
    view, For, IntoView, NodeRef, RwSignal, SignalGet, SignalGetUntracked, SignalSet, SignalWith,
    SignalWithUntracked, WriteSignal,
};
use leptos_use::use_intersection_observer;

use crate::ui::card_view::CardView;

#[derive(Debug, Default, Clone, Copy)]
struct CardFilter {
    name: RwSignal<String>,
    text: RwSignal<String>,
}

impl CardFilter {
    fn matches(&self, card: &Card) -> bool {
        if self
            .name
            .with(|name| !name.is_empty() && !card.name.to_ascii_lowercase().contains(name))
        {
            return false;
        }

        if self
            .text
            .with(|text| !text.is_empty() && !card.search_text.contains(text))
        {
            return false;
        }

        true
    }
}

#[derive(Clone, Copy)]
struct ScrollReset {
    set_pages: WriteSignal<usize>,
    scroll_area: NodeRef<html::Div>,
}

impl ScrollReset {
    fn reset(&self) {
        let area = self.scroll_area.get().unwrap();
        area.set_scroll_top(0);
        self.set_pages.set(1);
    }
}

#[component]
#[must_use]
pub fn FilterInput(
    placeholder: &'static str,
    map: fn(String) -> String,
    filter: RwSignal<String>,
) -> impl IntoView {
    let node_ref = create_node_ref();
    let reset = expect_context::<ScrollReset>();

    view! {
        <input
            type="text"
            placeholder=placeholder
            ref=node_ref
            on:input=move |_| {
                let input = node_ref.get().unwrap();
                reset.reset();
                filter.set(map(input.value()));
            }
        />
    }
}

#[component]
#[must_use]
pub fn CardSearch() -> impl IntoView {
    const PAGE_SIZE: usize = 50;

    let cards = expect_context::<CardData>();
    let filter = CardFilter::default();
    let filtered_cards = create_memo(move |_| {
        cards
            .entries()
            .filter(move |(_, card)| filter.matches(card))
            .map(|(id, _)| id)
            .collect::<Vec<_>>()
    });

    let (pages, set_pages) = create_signal(1);
    let paginated_cards = move || {
        filtered_cards.with(|cards| {
            cards
                .iter()
                .copied()
                .take(pages.get() * PAGE_SIZE)
                .collect::<Vec<_>>()
        })
    };

    // Automatic pagination
    let scroll_end_ref = create_node_ref::<html::Div>();
    use_intersection_observer(scroll_end_ref, move |entries, _| {
        let filtered_count = filtered_cards.with_untracked(Vec::len);
        let current_capacity = pages.get_untracked() * PAGE_SIZE;
        if entries[0].is_intersecting() && filtered_count > current_capacity {
            set_pages.set(pages.get_untracked() + 1);
        }
    });

    let scroll_area_ref = create_node_ref();
    provide_context(ScrollReset {
        set_pages,
        scroll_area: scroll_area_ref,
    });

    view! {
        <div class="card-search">
            <div class="card-search-params">
                <FilterInput placeholder="Name" map=|s| s.to_ascii_lowercase() filter=filter.name />
                <FilterInput
                    placeholder="Description"
                    map=|s| s.to_ascii_lowercase()
                    filter=filter.text
                />
            </div>

            <div class="card-list" ref=scroll_area_ref>
                <For
                    each=paginated_cards
                    key=|id| *id
                    children=move |id| {
                        view! { <CardView id=id /> }
                    }
                />

                <div style="position:relative; top: -20rem;" ref=scroll_end_ref></div>
            </div>
        </div>
    }
}
