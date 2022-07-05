use std::rc::Rc;

use yew::prelude::*;

use super::{layout::Topbar, resource_browser::ResourceBrowser};

use crate::yew::contexts::{resources::ResourcesProvider, todos::TodosProvider};

#[derive(Debug, PartialEq, Properties)]
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

#[function_component]
pub fn App(props: &AppProps) -> Html {
    let counter = use_state(|| 0);

    let inc = {
        let counter = counter.clone();

        move |_| {
            let value = *counter + 1;
            counter.set(value);
        }
    };

    let dec = {
        let counter = counter.clone();

        move |_| {
            let value = *counter - 1;
            counter.set(value);
        }
    };

    html! {
        <ResourcesProvider token={props.token.clone()}>
            <TodosProvider token={props.token.clone()}>
                <div class="h-full w-full flex flex-col flex-shrink-0">
                    <Topbar />
                    <div class="flex flex-row flex-grow h-full text-white">
                        <div class="w-[300px] border-r border-[#101010] overflow-y-auto p-2"><ResourceBrowser  /></div>
                        <div class="flex-grow flex flex-col">
                            <div class="flex-grow border-b border-[#101010] p-2">{"Editor"}</div>
                            <div class="h-[200px] p-2">
                                <div>{"Misc"}</div>
                                <div class="flex flex-row space-x-2">
                                <button onclick={dec} type="button">{"-1"}</button>
                                <p>{ *counter }</p>
                                <button onclick={inc} type="button">{"+1"}</button>
                                </div>
                            </div>
                            </div>
                        <div class="w-[300px] border-l border-[#101010] p-2">{"Property Grid"}</div>
                    </div>
                </div>
            </TodosProvider>
        </ResourcesProvider>
    }
}
