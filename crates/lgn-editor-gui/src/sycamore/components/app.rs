use std::rc::Rc;

use sycamore::prelude::*;

use crate::{sycamore::hooks::create_async_data, types::TodosRequest};

use super::{layout::Topbar, resource_browser::ResourceBrowser};

#[derive(Prop)]
pub struct AppProps {
    token: Rc<String>,
}

impl AppProps {
    pub fn new(token: String) -> Self {
        Self {
            token: Rc::new(token),
        }
    }
}

#[component]
pub fn App<G: Html>(cx: Scope, props: AppProps) -> View<G> {
    let counter = create_signal(cx, 0);

    let todos = create_async_data::<TodosRequest>(cx, props.token);

    provide_context(cx, todos);

    let inc = {
        move |_| {
            let value = *counter.get() + 1;

            counter.set(value);
        }
    };

    let dec = {
        move |_| {
            let value = *counter.get() - 1;

            counter.set(value);
        }
    };

    view! { cx,
        div(class="h-full w-full flex flex-col flex-shrink-0") {
            Topbar {}
            div(class="flex flex-row flex-grow h-full text-white") {
                div(class="w-[300px] border-r border-[#101010] overflow-y-auto p-2") {
                    ResourceBrowser {}
                }
                div(class="flex-grow flex flex-col text-white") {
                    div(class="flex-grow border-b border-[#101010] p-2") { "Editor" }
                    div(class="h-[200px] p-2") {
                        div { "Misc" }
                        div(class="flex flex-row space-x-2") {
                            button(on:click=dec, type="button") { "-1" }
                            p { (counter.get()) }
                            button(on:click=inc, type="button") { "+1" }
                        }
                    }
                }
                div(class="w-[300px] border-l border-[#101010] p-2") {
                    "Property Grid"
                }
            }
        }
    }
}
