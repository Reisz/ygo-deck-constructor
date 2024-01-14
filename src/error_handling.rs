use std::{error::Error, fmt};

use wasm_bindgen::JsValue;
use web_sys::js_sys::JsString;

#[macro_export]
macro_rules! print_error {
    ($($arg: expr),*) => {{
        let message = format!($($arg),*);
        gloo_dialogs::alert(&message);
    }}
}

#[derive(Debug)]
pub struct JsException(JsValue);

impl fmt::Display for JsException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "JavaScript exception: {}",
            JsString::from(self.0.clone())
        )
    }
}

impl Error for JsException {}

impl From<JsValue> for JsException {
    fn from(value: JsValue) -> Self {
        Self(value)
    }
}
