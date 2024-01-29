use common::card::CardLimit;
use leptos::{html, view, For, IntoView, SignalWith};

use crate::deck_part::DeckPart;

use super::Tool;

fn limit_name(limit: CardLimit) -> &'static str {
    match limit {
        CardLimit::Banned => "banned",
        CardLimit::Limited => "limited",
        CardLimit::SemiLimited => "semi-limited",
        CardLimit::Unlimited => "unlimited",
    }
}

#[derive(Debug, Default)]
pub struct ErrorList {
    totals: [usize; 3],
    errors: Vec<String>,
}

impl Tool for ErrorList {
    fn init() -> Self {
        Self::default()
    }

    fn fold(&mut self, entry: &super::CardDeckEntry) {
        let playing_part = if entry.card.card_type.is_extra_deck_monster() {
            DeckPart::Extra
        } else {
            DeckPart::Main
        };

        self.totals[playing_part as usize] += entry.playing;
        self.totals[DeckPart::Side as usize] += entry.side;

        let count = entry.playing + entry.side;
        let limit = entry.card.limit.count();

        if count > limit.into() {
            self.errors.push(format!(
                "Card \"{name}\" appears {count} times, but is {term} ({limit})",
                name = entry.card.name,
                term = limit_name(entry.card.limit)
            ));
        }
    }

    fn finish(&mut self) {
        for part in DeckPart::iter() {
            let len = self.totals[part as usize];

            if len < part.min().into() {
                self.errors.push(format!(
                    "{part} deck contains less than {} cards ({len})",
                    part.min(),
                ));
            } else if len > part.max().into() {
                self.errors.push(format!(
                    "{part} deck contains more than {} cards ({len})",
                    part.max(),
                ));
            }
        }
    }

    fn view(data: impl SignalWith<Value = Self> + Copy + 'static) -> impl IntoView {
        move || {
            data.with(|data| {
                if data.errors.is_empty() {
                    ().into_view()
                } else {
                    let errors = data.errors.clone();
                    view! {
                        <div>
                            <h3>"Errors"</h3>
                            <ul class="errors">
                                <For
                                    each=move || errors.clone()
                                    key=Clone::clone
                                    children=move |error| { html::li().child(error) }
                                />
                            </ul>
                        </div>
                    }
                    .into_view()
                }
            })
        }
    }
}
