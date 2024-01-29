mod error_list;
mod type_graph;

use common::{card::Card, card_data::CardData};
use leptos::{component, create_memo, expect_context, view, IntoView, Memo, RwSignal, SignalWith};

use crate::deck::{Deck, PartType};

struct CardDeckEntry {
    card: &'static Card,
    playing: usize,
    side: usize,
}

trait Tool {
    fn init() -> Self;
    fn fold(&mut self, entry: &CardDeckEntry);
    fn finish(&mut self) {}

    fn view(data: impl SignalWith<Value = Self> + Copy + 'static) -> impl IntoView;
}

/// Circumvent the equality check of `leptos::Memo`.
///
/// This is to prevent a superfluous equality check on a large tuple right before it gets split
/// again.
struct NoEq<T>(T);

impl<T> PartialEq for NoEq<T> {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

/// Slice for splitting the memoized calculation result of the `tools` macro.
struct Slice<T: 'static, U> {
    memo: Memo<NoEq<T>>,
    getter: fn(&T) -> &U,
}

impl<T: 'static, U> Clone for Slice<T, U> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: 'static, U> Copy for Slice<T, U> {}

impl<T: 'static, U> SignalWith for Slice<T, U> {
    type Value = U;

    fn with<O>(&self, f: impl FnOnce(&Self::Value) -> O) -> O {
        self.memo.with(move |data| f((self.getter)(&data.0)))
    }

    fn try_with<O>(&self, f: impl FnOnce(&Self::Value) -> O) -> Option<O> {
        Some(self.with(f))
    }
}

/// Variadic expansion of Tool implementations
///
/// Folds all tool calculations inside a large tuple, so the deck only has to be iterated once for
/// all tools.
macro_rules! tools {
    ($($ty:ty),*) => {
        type Data = ($($ty,)*);

        fn init() -> Data {
            ($(<$ty>::init(),)*)
        }

        fn fold(mut data: Data, entry: CardDeckEntry) -> Data {
            $({
                type _Tmp = $ty;
                data.${index()}.fold(&entry);
            })*
            data
        }

        fn finish(data: &mut Data) {
            $({
                type _Tmp = $ty;
                data.${index()}.finish();
            })*
        }

        fn view(memo: Memo<NoEq<Data>>) -> impl IntoView {
            ($(<$ty>::view(Slice{memo, getter: |data| &data.${index()}}),)*)
        }
    }
}

tools!(error_list::ErrorList, type_graph::TypeGraph);

#[component]
#[must_use]
pub fn Tools() -> impl IntoView {
    let cards = expect_context::<&'static CardData>();
    let deck = expect_context::<RwSignal<Deck>>();

    let data = create_memo(move |_| {
        deck.with(move |deck| {
            let mut data = deck
                .entries()
                .map(|entry| CardDeckEntry {
                    card: &cards[entry.id()],
                    playing: entry.count(PartType::Playing),
                    side: entry.count(PartType::Side),
                })
                .fold(init(), fold);
            finish(&mut data);
            NoEq(data)
        })
    });

    view! { <div class="tools">{view(data)}</div> }
}
