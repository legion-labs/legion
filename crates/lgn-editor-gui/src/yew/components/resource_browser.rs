use std::rc::Rc;

use yew::prelude::*;

use crate::{
    types::ResourceEntry,
    utils::tree::TreeVisitorMut,
    yew::{
        contexts::{resources::ResourcesContext, todos::TodosContext},
        hooks::AsyncData,
    },
};

struct ExampleResourceEntryTreeVisitor {
    html: Vec<Html>,
}

impl TreeVisitorMut<String, Rc<ResourceEntry>> for ExampleResourceEntryTreeVisitor {
    fn visit_value_mut(&mut self, entry: &Rc<ResourceEntry>, depth: u8) {
        self.html
            .push(html! { <Resource {depth} entry={entry.clone()} /> });
    }
}

#[derive(Debug, PartialEq, Properties)]
struct ResourceProps {
    depth: u8,
    entry: Rc<ResourceEntry>,
}

#[function_component]
fn Resource(props: &ResourceProps) -> Html {
    let in_edition = use_state(|| false);

    let onkeyup = {
        let in_edition = in_edition.clone();

        move |event: KeyboardEvent| {
            if &event.key() == "F2" {
                in_edition.set(true);
            }

            if &event.key() == "Escape" {
                in_edition.set(false);
            }
        }
    };

    let entry_html = match &*props.entry {
        ResourceEntry::Root => html! { <div>{"root"}</div> },
        ResourceEntry::Folder { name, .. } => {
            let style = format!("padding-left: {}px", 16 * props.depth);

            html! { <div {style}> if *in_edition { {"*"} } {name}</div> }
        }
        ResourceEntry::Entry { name, value, .. } => {
            let style = format!("padding-left: {}px", 16 * props.depth);
            let title = value.path.clone();

            html! { <div {style} {title}> if *in_edition { {"*"} } {name}</div> }
        }
    };

    html! {
        <div {onkeyup} tabindex="-1">
            {entry_html}
        </div>
    }
}

#[function_component]
pub fn ResourceBrowser() -> Html {
    let mut visitor = ExampleResourceEntryTreeVisitor { html: Vec::new() };

    let todos = use_context::<TodosContext>().unwrap();

    let _todos = match *todos {
        AsyncData::Data(ref todos) => {
            let todos = todos.iter().map(|todo| {
                html! {
                    <div>{&todo.title}</div>
                }
            });

            html! { <div>{ for todos }</div> }
        }
        AsyncData::Init => html! { <div>{"Loading"}</div> },
        AsyncData::Error(ref error) => html! { <div>{"Error: "}{error}</div> },
    };

    let todos = html! {};

    let resources = use_context::<ResourcesContext>().unwrap();

    if let AsyncData::Data(ref resources) = *resources {
        visitor.visit_tree_mut(&resources.clone().into(), 0);
    };

    html! { <div><div>{visitor.html}</div><div>{todos}</div></div> }
}
