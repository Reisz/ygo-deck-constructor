use std::fmt;

pub trait TextEncoding: Sized {
    fn encode(&self, writer: &mut impl fmt::Write) -> fmt::Result;
    fn decode(text: &str) -> Option<Self>;

    fn encode_string(&self) -> String {
        let mut result = String::new();
        self.encode(&mut result).unwrap(/* Write for String should never fail */);
        result
    }
}
