use common::{card_data::CardData, deck::PartType, deck_part::DeckPart};
use leptos::{html, prelude::*};

use crate::deck::Deck;

use super::Tool;

pub struct ErrorList;

impl Tool for ErrorList {
    fn init() -> Self {
        Self
    }

    fn view(&self, deck: Signal<Deck>) -> AnyView {
        let cards = expect_context::<CardData>();

        let errors = Memo::new(move |_| {
            let mut totals = [0; 3];
            let mut limit_exceeded = 0;

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

                    if playing + side > card.limit.count() {
                        limit_exceeded += 1;
                    }
                }
            });

            let mut errors = vec![];

            if limit_exceeded > 0 {
                errors.push(format!(
                    "Too many copies of {limit_exceeded} card{}",
                    if limit_exceeded > 1 { "s" } else { "" }
                ));
            }

            for part in DeckPart::iter() {
                let len = totals[part as usize];

                if len < part.min() {
                    errors.push(format!(
                        "{part} deck contains less than {} cards",
                        part.min(),
                    ));
                } else if len > part.max() {
                    errors.push(format!(
                        "{part} deck contains more than {} cards",
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
        .into_any()
    }
}
