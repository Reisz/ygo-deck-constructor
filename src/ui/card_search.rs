use common::card::{Card, CardData};
use leptos::{
    component, create_node_ref, create_signal, expect_context, html, view, For, IntoView,
    SignalUpdate, SignalWith,
};

use crate::ui::card_view::CardView;

#[derive(Debug, Default, Clone)]
struct CardFilter {
    name: String,
    text: String,
}

impl CardFilter {
    fn matches(&self, card: &Card) -> bool {
        if !self.name.is_empty() && !card.name.to_ascii_lowercase().contains(&self.name) {
            return false;
        }

        if !self.text.is_empty() && !card.description.to_ascii_lowercase().contains(&self.text) {
            return false;
        }

        true
    }

    fn set_name_filter(&mut self, filter: &str) {
        self.name = filter.to_ascii_lowercase();
    }

    fn set_text_filter(&mut self, filter: &str) {
        self.text = filter.to_ascii_lowercase();
    }
}

#[component]
#[must_use]
pub fn CardSearch() -> impl IntoView {
    let cards = expect_context::<&'static CardData>();

    let (filter, set_filter) = create_signal(CardFilter::default());

    let card_iter = move || {
        cards
            .iter()
            .filter(move |(_, card)| filter.with(|filter| filter.matches(card)))
            .take(50) // TODO: implement paging
    };

    let name_input_ref = create_node_ref::<html::Input>();
    let text_input_ref = create_node_ref::<html::Input>();
    view! {
        <div class="card-search">
            <div class="card-search-params">
                <input
                    type="text"
                    placeholder="Name"
                    node_ref=name_input_ref
                    on:input=move |_| {
                        let input = name_input_ref.get().unwrap();
                        set_filter.update(|filter| filter.set_name_filter(&input.value()));
                    }
                />

                <input
                    type="text"
                    placeholder="Description"
                    node_ref=text_input_ref
                    on:input=move |_| {
                        let input = text_input_ref.get().unwrap();
                        set_filter.update(|filter| filter.set_text_filter(&input.value()));
                    }
                />

            </div>

            <div class="card-list">
                <For
                    each=card_iter
                    key=|(id, _)| *id
                    children=move |(id, _)| {
                        view! { <CardView id=*id/> }
                    }
                />

            </div>
        </div>
    }
}
