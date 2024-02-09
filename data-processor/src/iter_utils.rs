use std::{fmt::Display, sync::Mutex};

use indicatif::{ParallelProgressIterator, ProgressStyle};
use rayon::iter::{
    FromParallelIterator, IndexedParallelIterator, IntoParallelIterator, ParallelIterator,
};

use crate::print_err;

pub trait IntoParProgressIterator {
    type Item: Send;

    fn into_par_progress_iter(self) -> impl ParallelIterator<Item = Self::Item>;
}

impl<T> IntoParProgressIterator for T
where
    T: IntoParallelIterator,
    T::Iter: IndexedParallelIterator,
{
    type Item = T::Item;

    fn into_par_progress_iter(self) -> impl ParallelIterator<Item = Self::Item> {
        let style = ProgressStyle::with_template("{bar} {human_pos}/{human_len} ({eta} remaining)")
            .unwrap();
        self.into_par_iter().progress_with_style(style)
    }
}

pub trait CollectParallelWithoutErrors<T: Send, E> {
    fn collect_without_errors<B>(self) -> B
    where
        B: FromParallelIterator<T>;
}

impl<I, T: Send, E: Send + Display> CollectParallelWithoutErrors<T, E> for I
where
    I: ParallelIterator<Item = Result<T, E>>,
{
    fn collect_without_errors<B>(self) -> B
    where
        B: FromParallelIterator<T>,
    {
        let errors = Mutex::new(Vec::new());
        let result = self
            .filter_map(|result| match result {
                Ok(value) => Some(value),
                Err(err) => {
                    errors.lock().unwrap().push(err);
                    None
                }
            })
            .collect::<B>();

        for err in errors.into_inner().unwrap() {
            print_err!("{err}");
        }

        result
    }
}
