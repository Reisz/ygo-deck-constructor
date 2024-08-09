mod error_list;
mod graphs;

use leptos::{component, expect_context, view, IntoView, RwSignal, Signal, View};

use crate::deck::Deck;

trait Tool {
    fn init() -> Self
    where
        Self: Sized;

    fn view(&self, deck: Signal<Deck>) -> View;
}

struct ToolManager(Vec<Box<dyn Tool>>);

impl ToolManager {
    fn new() -> Self {
        Self(vec![])
    }

    fn add<T: Tool + 'static>(&mut self) {
        self.0.push(Box::new(T::init()));
    }

    fn view(&self) -> impl IntoView {
        let deck = expect_context::<RwSignal<Deck>>();
        self.0
            .iter()
            .map(|tool| tool.view(deck.into()))
            .collect::<Vec<_>>()
    }
}

#[component]
#[must_use]
pub fn Tools() -> impl IntoView {
    let mut tools = ToolManager::new();

    tools.add::<error_list::ErrorList>();
    tools.add::<graphs::TypeGraph>();
    tools.add::<graphs::ExtraTypeGraph>();
    tools.add::<graphs::LevelGraph>();

    view! { <div class="tools">{tools.view()}</div> }
}
