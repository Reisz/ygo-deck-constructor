mod error_list;

use error_list::ErrorList;
use leptos::{component, view, IntoView};

#[component]
#[must_use]
pub fn Tools() -> impl IntoView {
    view! {
        <div class="tools">
            <ErrorList/>
        </div>
    }
}
