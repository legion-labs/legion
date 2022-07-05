use yew::prelude::*;

#[function_component]
pub fn Topbar() -> Html {
    html! {
        <div class="h-8 text-white bg-[#222] flex justify-between items-center px-4 flex-shrink-0 border-b border-[#101010]">
          <div>{"Window"}</div>
          <div>{"Legion Sample project"}</div>
          <div>{"User"}</div>
        </div>
    }
}
