use common::{card::CardType, card_data::CardData};
use leptos::{expect_context, view, IntoView, RwSignal, SignalWith, View};

use crate::{deck::Deck, deck_part::DeckPart};

use super::Tool;

fn make_counter(predicate: fn(&CardType) -> bool) -> impl Fn() -> usize + Copy {
    let cards = expect_context::<&'static CardData>();
    let deck = expect_context::<RwSignal<Deck>>();

    move || {
        deck.with(|deck| {
            deck.iter_part(cards, DeckPart::Main)
                .filter_map(|(id, count)| predicate(&cards[id].card_type).then_some(count))
                .sum::<usize>()
        })
    }
}

pub struct TypeGraph;

impl Tool for TypeGraph {
    fn view(&self) -> View {
        let monsters = make_counter(CardType::is_monster);
        let spells = make_counter(CardType::is_spell);
        let traps = make_counter(CardType::is_trap);

        view! {
            <div>
                <h3>"Card Types"</h3>
                <svg class="graph">
                    <svg viewBox="0 0 40 30" preserveAspectRatio="none">
                        <path d="M10 0 V30 M20 0 V30 M30 0 V30" class="helper"></path>
                        <rect y="1" width=monsters class="bar monster effect"></rect>
                        <rect y="11" width=spells class="bar spell"></rect>
                        <rect y="21" width=traps class="bar trap"></rect>
                        <path d="M0 0 V30" class="axis"></path>
                    </svg>
                    <text y="16.66%" font-size="0.9rem" class="label">
                        {monsters}
                    </text>
                    <text y="50%" font-size="0.9rem" class="label">
                        {spells}
                    </text>
                    <text y="83.33%" font-size="0.9rem" class="label">
                        {traps}
                    </text>
                </svg>
            </div>
        }
        .into_view()
    }
}
