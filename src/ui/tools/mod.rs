mod error_list;
mod type_graph;

use error_list::ErrorList;
use leptos::{component, view, IntoView};
use type_graph::TypeGraph;

#[component]
#[must_use]
pub fn Tools() -> impl IntoView {
    view! {
        <div class="tools">
            <ErrorList/>
            <TypeGraph/>
        </div>
    }
}
