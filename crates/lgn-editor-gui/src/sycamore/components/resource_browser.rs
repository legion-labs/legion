use sycamore::prelude::*;

use crate::{errors::Error, sycamore::hooks::AsyncData, types::Todo};

#[component]
pub fn ResourceBrowser<G: Html>(cx: Scope) -> View<G> {
    let todos = use_context::<RcSignal<AsyncData<Vec<Todo>, Error>>>(cx);

    let todos_list = create_memo(cx, || {
        if let AsyncData::Data(ref todos) = *todos.get() {
            todos.clone()
        } else {
            Vec::new()
        }
    });

    view! { cx,
        (if let AsyncData::Data(_) = *todos.get() {
            view! { cx,
                div {
                    Indexed {
                        iterable: todos_list,
                        view: |cx, todo| view! { cx, div { (todo.title) } }
                    }
                }
            }
        } else if let AsyncData::Init = *todos.get() {
            view! { cx, div { "Loading" } }
        } else if let AsyncData::Error(_) = *todos.get() {
            view! { cx, div { "Error" } }
        } else {
            view! { cx, div { "?" } }
        })
    }
}
