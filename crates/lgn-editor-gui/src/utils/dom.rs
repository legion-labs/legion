use url::Url;
use wasm_bindgen::JsCast;
use web_sys::{HtmlDocument, Window};

use crate::errors::{Error, Result};

pub fn get_window() -> Result<Window> {
    web_sys::window().ok_or_else(|| Error::Js("window object not found".into()))
}

fn html_document() -> Result<HtmlDocument> {
    get_window()?
        .document()
        .ok_or_else(|| Error::Js("document object not found".into()))?
        .dyn_into::<HtmlDocument>()
        .map_err(|_| Error::Js("document object couldn't be cast to html document".into()))
}

pub fn set_cookie(
    name: impl AsRef<str>,
    value: impl AsRef<str>,
    max_age: Option<u64>,
) -> Result<()> {
    let html_document = html_document()?;

    let location = get_current_url()?;

    let domain = location
        .host_str()
        .ok_or_else(|| Error::Js("href not found in location".into()))?;

    let mut cookie = format!(
        "{name}={value};domain={domain};path=/;samesite=strict;secure;",
        name = name.as_ref(),
        value = value.as_ref(),
        domain = domain,
    );

    if let Some(max_age) = max_age {
        cookie.push_str(&format!("max-age={max_age}", max_age = max_age));
    }

    html_document
        .set_cookie(&cookie)
        .map_err(|_| Error::Js("couldn't set cookie".into()))
}

pub fn get_cookie(name: impl AsRef<str>) -> Result<Option<String>> {
    let cookie = html_document()?
        .cookie()
        .map_err(|_| Error::Js("couldn't get cookies from document".into()))?;

    for cookie in cookie.split(';') {
        let parts = cookie.split('=').collect::<Vec<_>>();

        if parts.len() != 2 || parts.get(0).map(|part| part.trim()) != Some(name.as_ref()) {
            continue;
        }

        return Ok(parts.get(1).map(|part| part.trim().to_string()));
    }

    Ok(None)
}

pub fn get_current_url() -> Result<Url> {
    get_window()?
        .location()
        .href()
        .map_err(|_| Error::Js("href not found in location".into()))?
        .parse()
        .map_err(|_| Error::Js("couldn't parse current url".into()))
}

pub fn set_window_location(url: &Url) -> Result<()> {
    let location = get_window()?.location();

    location
        .set_href(url.as_ref())
        .map_err(|_| Error::Js("couldn't set location url".into()))?;

    Ok(())
}
