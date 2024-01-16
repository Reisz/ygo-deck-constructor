mod error_list;
mod type_graph;

use error_list::ErrorList;
use leptos::{component, view, IntoView, View};
use type_graph::TypeGraph;

trait Tool {
    fn view(&self) -> View;
}

#[component]
#[must_use]
pub fn Tools() -> impl IntoView {
    let tools: &[&'static dyn Tool] = &[&ErrorList, &TypeGraph];
    view! { <div class="tools">{tools.iter().map(|view| view.view()).collect::<Vec<_>>()}</div> }
}
