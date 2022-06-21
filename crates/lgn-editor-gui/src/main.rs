use log::{debug, error, Level};
use wasm_bindgen_futures::spawn_local;

use crate::utils::auth::get_redirect_uri;
use crate::utils::{
    auth::{get_authorization_url, get_code_in_url, get_token_set},
    dom::{get_cookie, get_current_url, set_cookie, set_window_location},
};

mod errors;
#[cfg(feature = "sycamore")]
mod sycamore;
mod types;
mod utils;
#[cfg(feature = "yew")]
mod yew;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

static COOKIE_NAME: &str = "editor_yew_local";

fn main() {
    spawn_local(async {
        console_log::init_with_level(Level::Trace).unwrap();

        let current_url = get_current_url().unwrap();

        match get_code_in_url(&current_url) {
            Some(code) => {
                debug!("Code {}", code);

                match get_token_set(&code).await {
                    Ok(token_set) => {
                        debug!("Token set {:#?}", token_set);

                        set_cookie(
                            COOKIE_NAME,
                            token_set.access_token,
                            Some(token_set.expires_in),
                        )
                        .unwrap();

                        // TODO: Use yew redirections?
                        set_window_location(&get_redirect_uri().unwrap()).unwrap();
                    }
                    Err(err) => {
                        error!("Impossible to get token set: {}", err);
                    }
                }
            }
            None => {
                let token = match get_cookie(COOKIE_NAME).unwrap() {
                    Some(token) => token,
                    None => {
                        let authorization_url = get_authorization_url().unwrap();

                        debug!("Authorization url {}", authorization_url);

                        set_window_location(&authorization_url).unwrap();

                        return;
                    }
                };

                #[cfg(feature = "yew")]
                ::yew::Renderer::<crate::yew::components::app::App>::with_props(
                    crate::yew::components::app::AppProps::new(token),
                )
                .render();

                #[cfg(feature = "sycamore")]
                ::sycamore::render(|cx| {
                    crate::sycamore::components::app::App(
                        cx,
                        crate::sycamore::components::app::AppProps::new(token),
                    )
                });
            }
        }
    });
}
