use common::{card::CardLimit, card_data::CardData};
use leptos::{expect_context, html, view, For, IntoView, RwSignal, SignalWith, View};

use crate::{
    deck::{Deck, PartType},
    deck_part::DeckPart,
};

use super::Tool;

pub struct ErrorList;

impl Tool for ErrorList {
    fn view(&self) -> View {
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
                    let card = &cards[entry.id()];
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

        let view = move || {
            if errors().is_empty() {
                ().into_view()
            } else {
                view! {
                    <div>
                        <h3>"Errors"</h3>
                        <ul class="errors">
                            <For
                                each=errors
                                key=Clone::clone
                                children=move |error| { html::li().child(error) }
                            />
                        </ul>
                    </div>
                }
                .into_view()
            }
        };
        view.into_view()
    }
}
