use std::fmt::Write;

use common::card::{CardType, MonsterStats, MonsterType};
use leptos::{component, view, CollectView, IntoSignal, IntoView, Signal, SignalWith};

use super::{CardDeckEntry, Tool};

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
#[allow(clippy::needless_lifetimes)] // falso positive
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
    for elem in helper_positions.map(Some).intersperse(None) {
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
        let bars = [
            GraphBar::new(move || data.with(|data| data.monster), "monster effect"),
            GraphBar::new(move || data.with(|data| data.spell), "spell"),
            GraphBar::new(move || data.with(|data| data.trap), "trap"),
        ];

        view! {
            <div>
                <h3>"Card Types"</h3>
                <Graph extent=40 bars=&bars/>
            </div>
        }
    }
}

#[derive(Debug, Default)]
pub struct ExtraTypeGraph {
    fusion: usize,
    synchro: usize,
    xyz: usize,
    link: usize,
}

impl Tool for ExtraTypeGraph {
    fn init() -> Self {
        Self::default()
    }

    fn fold(&mut self, entry: &CardDeckEntry) {
        if let CardType::Monster { stats, .. } = &entry.card.card_type {
            let counter = match stats {
                MonsterStats::Normal {
                    monster_type: Some(MonsterType::Fusion),
                    ..
                } => &mut self.fusion,
                MonsterStats::Normal {
                    monster_type: Some(MonsterType::Synchro),
                    ..
                } => &mut self.synchro,
                MonsterStats::Normal {
                    monster_type: Some(MonsterType::Xyz),
                    ..
                } => &mut self.xyz,
                MonsterStats::Normal { .. } => return,
                MonsterStats::Link { .. } => &mut self.link,
            };

            *counter += entry.playing;
        }
    }

    fn view(data: impl SignalWith<Value = Self> + Copy + 'static) -> impl IntoView {
        let bars = [
            GraphBar::new(move || data.with(|data| data.fusion), "monster fusion"),
            GraphBar::new(move || data.with(|data| data.synchro), "monster synchro"),
            GraphBar::new(move || data.with(|data| data.xyz), "monster xyz"),
            GraphBar::new(move || data.with(|data| data.link), "monster link"),
        ];

        view! {
            <div>
                <h3>"Extra Deck Card Types"</h3>
                <Graph extent=15 spacing=5 bars=&bars/>
            </div>
        }
    }
}

#[derive(Debug, Default)]
pub struct LevelGraph {
    no_tribute: usize,
    one_tribute: usize,
    two_tributes: usize,
}

impl Tool for LevelGraph {
    fn init() -> Self {
        Self::default()
    }

    fn fold(&mut self, entry: &CardDeckEntry) {
        if let CardType::Monster {
            stats:
                MonsterStats::Normal {
                    level,
                    monster_type,
                    ..
                },
            ..
        } = &entry.card.card_type
        {
            if monster_type.is_some() {
                return;
            }

            let counter = match level {
                0..=4 => &mut self.no_tribute,
                5..=6 => &mut self.one_tribute,
                7.. => &mut self.two_tributes,
            };
            *counter += entry.playing;
        }
    }

    fn view(data: impl SignalWith<Value = Self> + Copy + 'static) -> impl IntoView {
        let bars = [
            GraphBar::with_label(
                move || data.with(|data| data.no_tribute),
                "monster",
                "0 - 4",
            ),
            GraphBar::with_label(
                move || data.with(|data| data.one_tribute),
                "monster",
                "5 - 6",
            ),
            GraphBar::with_label(move || data.with(|data| data.two_tributes), "monster", "7+"),
        ];

        view! {
            <div>
                <h3>"Monster Levels"</h3>
                <Graph extent=30 bars=&bars/>
            </div>
        }
    }
}
