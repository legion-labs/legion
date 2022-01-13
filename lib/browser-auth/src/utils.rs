use alloc::{borrow::Cow, format, string::ToString, vec::Vec};
use anyhow::anyhow;
use url::Url;
use wasm_bindgen::JsCast;
use web_sys::{window, HtmlDocument};

/// When the `console_error_panic_hook` feature is enabled, we can call the
/// `set_panic_hook` function at least once during initialization, and then
/// we will get better error messages if our code ever panics.
///
/// For more details see [here](https://github.com/rustwasm/console_error_panic_hook#readme).
pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

// This macro is kept for debugging for purpose even though it's not always used
/// A macro that's close enough to the `print` macro but works in the browser
#[macro_export]
macro_rules! log {
    ($($x:expr),+) => {
        web_sys::console::log_1(
            &wasm_bindgen::JsValue::from_str(
                alloc::format!($($x),+).as_str()
            )
        );
    };
}

/// A key/value representation of a cookie
pub(crate) type CookiePair<'a> = (&'a str, &'a str);

/// A simple [`Vec`] of [`CookiePair`]
pub(crate) struct Cookies<'a>(Vec<CookiePair<'a>>);

impl<'a> Cookies<'a> {
    /// Gets the cookie by key/name
    pub(crate) fn get(&self, name: &str) -> Option<&'a str> {
        self.0
            .iter()
            .find_map(|(key, value)| if *key == name { Some(*value) } else { None })
    }
}

/// Takes the cookie string typically returned by the [`HtmlDocument::cookie`] function
/// and turns it into a [`Cookies`] struct.
pub(crate) fn parse_cookies(document_cookie: &str) -> Cookies<'_> {
    Cookies(
        document_cookie
            .split(';')
            .filter_map(|cookie| {
                let parts = cookie.split('=').collect::<Vec<&str>>();

                match parts.as_slice() {
                    [key, value] => Some((key.trim(), value.trim())),
                    _ => None,
                }
            })
            .collect(),
    )
}

/// Takes an [`HtmlDocument`], a cookie key and value, and the amount of seconds before it expires.
pub(crate) fn document_set_cookie(
    document: &HtmlDocument,
    name: &str,
    value: &impl ToString,
    expires_in: u64,
) -> anyhow::Result<()> {
    document
        .set_cookie(&format!(
            "{}={};domain=localhost;path=/;max-age={};samesite=strict;secure;",
            name,
            value.to_string(),
            expires_in
        ))
        .map_err(|_error| anyhow!("Couldn't set {} cookie", name))
}

/// Gets the global `document` object as an [`HtmlDocument`].
pub(crate) fn get_document() -> HtmlDocument {
    let window = window().unwrap();

    window
        .document()
        .unwrap()
        .dyn_into::<HtmlDocument>()
        .unwrap()
}

/// Gets a query parameter value and pass it down to the provided function.
/// The function is not called if the value couldn't be found and an error is returned.
pub(crate) fn get_url_query_value<'s, F, V>(url: &'s Url, name: &str, mut f: F) -> anyhow::Result<V>
where
    F: FnMut(Cow<'s, str>) -> V,
{
    let value = url
        .query_pairs()
        .find_map(
            |(key, value)| {
                if key == name {
                    Some(f(value))
                } else {
                    None
                }
            },
        )
        .ok_or_else(|| anyhow!("Key {} couldn't be found in URL query string", name))?;

    Ok(value)
}

/// Similar to [`get_url_query_value`] but instead of taking a function
/// it returns the value as [`Cow`].
pub(crate) fn get_url_query_string_value<'s>(
    url: &'s Url,
    name: &str,
) -> anyhow::Result<Cow<'s, str>> {
    get_url_query_value(url, name, |value| value)
}
