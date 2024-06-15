use crate::print_err;

pub trait CollectWithoutErrors<T: Send, E> {
    fn collect_without_errors<B>(self) -> B
    where
        B: FromIterator<T>;
}

impl<I, T: Send, E: Into<anyhow::Error>> CollectWithoutErrors<T, E> for I
where
    I: Iterator<Item = Result<T, E>>,
{
    fn collect_without_errors<B>(self) -> B
    where
        B: FromIterator<T>,
    {
        let mut errors = Vec::new();
        let result = self
            .filter_map(|result| match result {
                Ok(value) => Some(value),
                Err(err) => {
                    errors.push(err);
                    None
                }
            })
            .collect::<B>();

        for err in errors {
            print_err(&err.into());
        }

        result
    }
}
