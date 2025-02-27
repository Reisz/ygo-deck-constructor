use common::{card::Card, card_data::CardData};
use leptos::{html, prelude::*};
use wasm_bindgen::{JsCast, closure::Closure};
use web_sys::js_sys;

use crate::ui::card_view::CardView;

#[derive(Debug, Default, Clone, Copy)]
struct CardFilter {
    name: RwSignal<String>,
    text: RwSignal<String>,
}

impl CardFilter {
    fn is_empty(&self) -> bool {
        self.name.with(String::is_empty) && self.text.with(String::is_empty)
    }

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
    pub callback: Callback<(), ()>,
}

#[component]
#[must_use]
pub fn FilterInput(
    placeholder: &'static str,
    map: fn(String) -> String,
    filter: RwSignal<String>,
) -> impl IntoView {
    let node_ref = NodeRef::new();
    let reset = expect_context::<ScrollReset>();

    view! {
        <input
            type="text"
            placeholder=placeholder
            node_ref=node_ref
            on:input=move |_| {
                let input = node_ref.get().unwrap();
                filter.set(map(input.value()));
                reset.callback.run(());
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
    let filtered_cards = Memo::new(move |_| {
        if filter.is_empty() {
            cards.staples().collect::<Vec<_>>()
        } else {
            cards
                .entries()
                .filter(move |(_, card)| filter.matches(card))
                .map(|(id, _)| id)
                .collect::<Vec<_>>()
        }
    });

    let (pages, set_pages) = signal(1);
    let paginated_cards = move || {
        filtered_cards.with(|cards| {
            cards
                .iter()
                .copied()
                .take(pages.get() * PAGE_SIZE)
                .collect::<Vec<_>>()
        })
    };

    let scroll_area_ref = NodeRef::<html::Div>::new();

    // Increase page count until the scroll buffer is sufficiently filled
    let adjust_pages = move || {
        let scroll_area = scroll_area_ref.get_untracked().unwrap();
        let card_count = filtered_cards.with_untracked(Vec::len);
        let mut pages = pages.get_untracked();

        let area_height = scroll_area.client_height();
        let offset = scroll_area.scroll_top() + area_height;

        while scroll_area.scroll_height() - offset <= area_height / 2
            && pages * PAGE_SIZE < card_count
        {
            pages += 1;
            set_pages.set(pages);
        }
    };

    // Provide a callback to reset scrolling when the search text changes
    provide_context(ScrollReset {
        callback: Callback::new(move |()| {
            let scroll_area = scroll_area_ref.get_untracked().unwrap();
            scroll_area.set_scroll_top(0);

            set_pages.set(0);
            adjust_pages();
        }),
    });

    // Adjust page count on resize
    scroll_area_ref.on_load(move |scroll_area| {
        let callback = move |_, _| adjust_pages();
        let callback = Closure::<dyn Fn(js_sys::Array, web_sys::ResizeObserver)>::new(callback)
            .into_js_value();
        let observer = web_sys::ResizeObserver::new(callback.as_ref().unchecked_ref()).unwrap();
        observer.observe(&scroll_area);
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

            <div class="card-list" node_ref=scroll_area_ref on:scroll=move |_| adjust_pages()>
                <For
                    each=paginated_cards
                    key=|id| *id
                    children=move |id| {
                        view! { <CardView id=id /> }
                    }
                />
            </div>
        </div>
    }
}
