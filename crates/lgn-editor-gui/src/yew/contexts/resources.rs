use std::rc::Rc;

use yew::prelude::*;

use crate::{
    errors::Error,
    types::{NextSearchToken, NextSearchTokenRequest},
    yew::hooks::{use_async_data, AsyncData},
};

pub type ResourcesContext = UseStateHandle<AsyncData<Rc<NextSearchToken>, Error>>;

#[derive(Properties, Debug, PartialEq)]
pub struct ResourcesProviderProps {
    #[prop_or_default]
    pub children: Children,
    pub token: Rc<String>,
}

#[function_component]
pub fn ResourcesProvider(props: &ResourcesProviderProps) -> Html {
    let resources = use_async_data::<NextSearchTokenRequest>(props.token.clone());

    html! {
        <ContextProvider<ResourcesContext> context={resources}>
            {props.children.clone()}
        </ContextProvider<ResourcesContext>>
    }
}
