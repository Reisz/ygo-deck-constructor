use common::card::CardType;
use leptos::{view, IntoView, SignalWith};

use super::{CardDeckEntry, Tool};

#[derive(Debug, Default)]
pub struct TypeGraph {
    monster: usize,
    spell: usize,
    trap: usize,
}

impl Tool for TypeGraph {
    fn init() -> Self {
        Self::default()
    }

    fn fold(&mut self, entry: &CardDeckEntry) {
        let counter = match entry.card.card_type {
            CardType::Monster { .. } => {
                if entry.card.card_type.is_extra_deck_monster() {
                    return;
                }

                &mut self.monster
            }
            CardType::Spell(_) => &mut self.spell,
            CardType::Trap(_) => &mut self.trap,
        };
        *counter += entry.playing;
    }

    fn view(data: impl SignalWith<Value = Self> + Copy + 'static) -> impl IntoView {
        let monsters = move || data.with(|data| data.monster);
        let spells = move || data.with(|data| data.spell);
        let traps = move || data.with(|data| data.trap);

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
    }
}
