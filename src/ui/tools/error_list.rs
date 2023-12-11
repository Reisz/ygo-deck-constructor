use common::card::{CardData, CardLimit};
use leptos::{component, expect_context, html, view, For, IntoView, RwSignal, SignalWith};

use crate::{
    deck::{Deck, PartType},
    deck_part::DeckPart,
};

#[component]
pub fn ErrorList() -> impl IntoView {
    let deck = expect_context::<RwSignal<Deck>>();
    let cards = expect_context::<&'static CardData>();

    let errors = move || {
        let mut errors = Vec::new();

        deck.with(|deck| {
            for part in DeckPart::iter() {
                let len = deck
                    .iter_part(cards, part)
                    .map(|(_, count)| count)
                    .sum::<usize>();

                if len < part.min().into() {
                    errors.push(format!(
                        "{part} deck contains less than {} cards ({len})",
                        part.min(),
                    ));
                } else if len > part.max().into() {
                    errors.push(format!(
                        "{part} deck contains more than {} cards ({len})",
                        part.max(),
                    ));
                }
            }

            for entry in deck.entries() {
                let card = &cards[&entry.id()];
                let count = entry.count(PartType::Playing) + entry.count(PartType::Side);

                let limit = card.limit.count();
                let term = match card.limit {
                    CardLimit::Banned => "banned",
                    CardLimit::Limited => "limited",
                    CardLimit::SemiLimited => "semi-limited",
                    CardLimit::Unlimited => "unlimited",
                };

                if count > limit.into() {
                    errors.push(format!(
                        "Card \"{}\" appears {count} times, but is {term} ({limit})",
                        card.name
                    ));
                }
            }
        });

        errors
    };

    view! {
        <h3>{move || { if errors().is_empty() { "No Errors" } else { "Errors" } }}</h3>
        <ul>
            <For each=errors key=Clone::clone children=move |error| { html::li().child(error) }/>
        </ul>
    }
}
