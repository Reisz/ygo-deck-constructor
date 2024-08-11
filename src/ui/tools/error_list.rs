use common::{card::CardLimit, card_data::CardData, deck::PartType, deck_part::DeckPart};
use leptos::{
    expect_context, html, view, For, IntoView, Memo, Show, Signal, SignalGet, SignalWith, View,
};

use crate::deck::Deck;

use super::Tool;

fn limit_name(limit: CardLimit) -> &'static str {
    match limit {
        CardLimit::Banned => "banned",
        CardLimit::Limited => "limited",
        CardLimit::SemiLimited => "semi-limited",
        CardLimit::Unlimited => "unlimited",
    }
}

pub struct ErrorList;

impl Tool for ErrorList {
    fn init() -> Self {
        Self
    }

    fn view(&self, deck: Signal<Deck>) -> View {
        let cards = expect_context::<&'static CardData>();

        let errors = Memo::new(move |_| {
            let mut totals = [0; 3];
            let mut errors = Vec::<String>::new();

            deck.with(|deck| {
                for entry in deck.entries() {
                    let card = &cards[entry.id()];
                    let playing = entry.count(PartType::Playing);
                    let side = entry.count(PartType::Side);

                    let playing_part = if card.card_type.is_extra_deck_monster() {
                        DeckPart::Extra
                    } else {
                        DeckPart::Main
                    };

                    totals[playing_part as usize] += playing;
                    totals[DeckPart::Side as usize] += side;

                    let count = playing + side;
                    let limit = card.limit.count();

                    if count > limit.into() {
                        errors.push(format!(
                            "Card \"{name}\" appears {count} times, but is {term} ({limit})",
                            name = card.name,
                            term = limit_name(card.limit)
                        ));
                    }
                }
            });

            for part in DeckPart::iter() {
                let len = totals[part as usize];

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

            errors
        });

        view! {
            <Show when=move || !errors.with(Vec::is_empty)>
                <div>
                    <h3>"Errors"</h3>
                    <ul class="errors">
                        <For
                            each=move || errors.get()
                            key=Clone::clone
                            children=move |error| { html::li().child(error) }
                        />
                    </ul>
                </div>
            </Show>
        }
        .into_view()
    }
}
