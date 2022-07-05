use sycamore::prelude::*;

#[component]
pub fn Topbar<G: Html>(cx: Scope) -> View<G> {
    view! { cx,
        div(class="h-8 text-white bg-[#222] flex justify-between items-center px-4 flex-shrink-0 border-b border-[#101010]") {
            div { "Window" }
            div { "Legion Sample project" }
            div { "User" }
        }
    }
}
