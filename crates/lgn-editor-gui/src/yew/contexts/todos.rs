use std::rc::Rc;

use yew::prelude::*;

use crate::{
    errors::Error,
    types::{Todo, TodosRequest},
    yew::hooks::{use_async_data, AsyncData},
};

pub type TodosContext = UseStateHandle<AsyncData<Rc<Vec<Todo>>, Error>>;

#[derive(Properties, Debug, PartialEq)]
pub struct TodosProviderProps {
    #[prop_or_default]
    pub children: Children,
    pub token: Rc<String>,
}

#[function_component]
pub fn TodosProvider(props: &TodosProviderProps) -> Html {
    let todos = use_async_data::<TodosRequest>(props.token.clone());

    html! {
        <ContextProvider<TodosContext> context={todos}>
            {props.children.clone()}
        </ContextProvider<TodosContext>>
    }
}
