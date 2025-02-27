use std::fmt::Write;

use common::{
    card::{CardType, MonsterStats, MonsterType},
    card_data::CardData,
    deck::PartType,
};
use itertools::intersperse;
use leptos::{
    CollectView, IntoSignal, IntoView, Memo, Signal, SignalWith, View, component, expect_context,
    view,
};

use crate::deck::Deck;

use super::Tool;

#[derive(Debug, Clone)]
struct GraphBar {
    width: Signal<usize>,
    class: &'static str,
    label: Option<&'static str>,
}

impl GraphBar {
    fn new(width: impl IntoSignal<Value = usize>, class: &'static str) -> Self {
        Self {
            width: width.into_signal(),
            class,
            label: None,
        }
    }

    fn with_label(
        width: impl IntoSignal<Value = usize>,
        class: &'static str,
        label: &'static str,
    ) -> Self {
        Self {
            width: width.into_signal(),
            class,
            label: Some(label),
        }
    }
}

#[must_use]
#[component]
#[allow(clippy::needless_lifetimes)] // false positive
fn Graph<'a, const N: usize>(
    extent: usize,
    #[prop(optional)] spacing: Option<usize>,
    bars: &'a [GraphBar; N],
) -> impl IntoView {
    let height = N * 10;
    let n: f64 = u32::try_from(N).unwrap().into();

    let helper_positions = (0..)
        .step_by(spacing.unwrap_or(10))
        .skip(1)
        .take_while(|pos| *pos < extent);

    let mut helper_path = String::new();
    for elem in intersperse(helper_positions.map(Some), None) {
        match elem {
            Some(pos) => write!(helper_path, "M{pos} 0 V{height}").unwrap(),
            None => helper_path.push(' '),
        }
    }

    let labels = bars.iter().enumerate().map(|(idx, bar)| {
        let idx: f64 = u32::try_from(idx).unwrap().into();
        let pos = ((1.0 + 2.0 * idx) / (2.0 * n)) * 100.0;
        let y = format!("{pos}%");

        let value = view! {
            <text y=&y font-size="0.9rem" class="label value backdrop">
                {bar.width}
            </text>
        };

        let label = bar.label.map(|label| {
            view! {
                <text text-anchor="end" x="100%" y=&y font-size="0.9rem" class="label">
                    {label}
                </text>
            }
        });

        (value, label)
    });

    let bars = bars.iter().enumerate().map(|(idx, bar)| {
        view! { <rect y=10 * idx + 1 width=bar.width class=format!("bar {}", bar.class)></rect> }
    });

    view! {
        <svg class="graph" height=format!("{}rem", n * 1.8)>
            <svg viewBox=format!("0 0 {extent} {height}") preserveAspectRatio="none">
                <path d=helper_path class="helper"></path>
                {bars.collect_view()}
                <path d=format!("M0 0 V{height}") class="axis"></path>
            </svg>
            {labels.collect_view()}
        </svg>
    }
}

pub struct TypeGraph;

#[derive(Default, PartialEq, Eq)]
struct TypeCounts {
    monster: usize,
    spell: usize,
    trap: usize,
}

impl Tool for TypeGraph {
    fn init() -> Self {
        Self
    }

    fn view(&self, deck: Signal<Deck>) -> View {
        let cards = expect_context::<CardData>();

        let counts = Memo::new(move |_| {
            let mut counts = TypeCounts::default();

            deck.with(|deck| {
                for entry in deck.entries() {
                    let card = &cards[entry.id()];
                    let counter = match card.card_type {
                        CardType::Monster { .. } => {
                            if card.card_type.is_extra_deck_monster() {
                                continue;
                            }

                            &mut counts.monster
                        }
                        CardType::Spell(_) => &mut counts.spell,
                        CardType::Trap(_) => &mut counts.trap,
                    };
                    *counter += usize::from(entry.count(PartType::Playing));
                }
            });

            counts
        });

        let bars = [
            GraphBar::new(
                move || counts.with(|counts| counts.monster),
                "monster effect",
            ),
            GraphBar::new(move || counts.with(|counts| counts.spell), "spell"),
            GraphBar::new(move || counts.with(|counts| counts.trap), "trap"),
        ];

        view! {
            <div>
                <h3>"Card Types"</h3>
                <Graph extent=40 bars=&bars />
            </div>
        }
        .into_view()
    }
}

pub struct ExtraTypeGraph;

#[derive(Default, PartialEq, Eq)]
struct ExtraTypeCounts {
    fusion: usize,
    synchro: usize,
    xyz: usize,
    link: usize,
}

impl Tool for ExtraTypeGraph {
    fn init() -> Self {
        Self
    }

    fn view(&self, deck: Signal<Deck>) -> View {
        let cards = expect_context::<CardData>();

        let counts = Memo::new(move |_| {
            let mut counts = ExtraTypeCounts::default();

            deck.with(|deck| {
                for entry in deck.entries() {
                    let card = &cards[entry.id()];
                    if let CardType::Monster { stats, .. } = &card.card_type {
                        let counter = match stats {
                            MonsterStats::Normal {
                                monster_type: Some(MonsterType::Fusion),
                                ..
                            } => &mut counts.fusion,
                            MonsterStats::Normal {
                                monster_type: Some(MonsterType::Synchro),
                                ..
                            } => &mut counts.synchro,
                            MonsterStats::Normal {
                                monster_type: Some(MonsterType::Xyz),
                                ..
                            } => &mut counts.xyz,
                            MonsterStats::Normal { .. } => continue,
                            MonsterStats::Link { .. } => &mut counts.link,
                        };

                        *counter += usize::from(entry.count(PartType::Playing));
                    }
                }
            });

            counts
        });

        let bars = [
            GraphBar::new(
                move || counts.with(|counts| counts.fusion),
                "monster fusion",
            ),
            GraphBar::new(
                move || counts.with(|counts| counts.synchro),
                "monster synchro",
            ),
            GraphBar::new(move || counts.with(|counts| counts.xyz), "monster xyz"),
            GraphBar::new(move || counts.with(|counts| counts.link), "monster link"),
        ];

        view! {
            <div>
                <h3>"Extra Deck Card Types"</h3>
                <Graph extent=15 spacing=5 bars=&bars />
            </div>
        }
        .into_view()
    }
}

pub struct LevelGraph;

#[derive(Default, PartialEq, Eq)]
struct LevelCounts {
    no_tribute: usize,
    one_tribute: usize,
    two_tributes: usize,
}

impl Tool for LevelGraph {
    fn init() -> Self {
        Self
    }

    fn view(&self, deck: Signal<Deck>) -> View {
        let cards = expect_context::<CardData>();

        let counts = Memo::new(move |_| {
            let mut counts = LevelCounts::default();

            deck.with(|deck| {
                for entry in deck.entries() {
                    let card = &cards[entry.id()];
                    if let CardType::Monster {
                        stats:
                            MonsterStats::Normal {
                                level,
                                monster_type,
                                ..
                            },
                        ..
                    } = &card.card_type
                    {
                        if monster_type.is_some() {
                            continue;
                        }

                        let counter = match level {
                            0..=4 => &mut counts.no_tribute,
                            5..=6 => &mut counts.one_tribute,
                            7.. => &mut counts.two_tributes,
                        };
                        *counter += usize::from(entry.count(PartType::Playing));
                    }
                }
            });

            counts
        });
        let bars = [
            GraphBar::with_label(
                move || counts.with(|counts| counts.no_tribute),
                "monster",
                "0 - 4",
            ),
            GraphBar::with_label(
                move || counts.with(|counts| counts.one_tribute),
                "monster",
                "5 - 6",
            ),
            GraphBar::with_label(
                move || counts.with(|counts| counts.two_tributes),
                "monster",
                "7+",
            ),
        ];

        view! {
            <div>
                <h3>"Monster Levels"</h3>
                <Graph extent=30 bars=&bars />
            </div>
        }
        .into_view()
    }
}
