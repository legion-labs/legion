<!--
@Component
Context menu meant to replace the default context menu in browsers.

The component alone doesn't do much apart from replacing the context menu
by a new, custom, one.

In order to add more entries to the context menu, one must first use
the `contextMenuStore` to register the new entries, and then, using
the `contextMenu` action let this component know what custom menu
the element must use.

Alongside the `type` and `label`, an `onClick` attribute must be
provided. This function will be called with the `close` function
that allows to close the context menu programmatically, and
a `payload` if provided to the `contextMenu` action. Payloads
will be stringified using `JSON.stringify` to be store in the _DOM_
and parsed using `JSON.parse`, it's therefore strongly discouraged
to pass a big object, or any non serializable object.

_In the future the `payload` could be forced to be a `string`._

_This component is expected to be mounted _once_ in the whole page._
`contextMenuStore.register` can be called as many times as needed,
and a `contextMenuStore.remove` function is also provided to cleanup unnecessary
context menu entries.

## Example

`./stores/myContextMenu.ts`

```typescript
import buildContextMenuStore from "@lgn/frontend/src/stores/contextMenu";

export type MyContextMenuName = "my-context-menu" | "my-other-context-menu";

export default buildContextMenuStore<MyContextMenuName>();
```

`./actions/myContextMenu.ts`

```typescript
import buildContextMenu from "@lgn/frontend/src/actions/contextMenu";
import { MyContextMenuName } from "../stores/myContextMenu";

export default buildContextMenu<MyContextMenuName>();
```

`./pages/MyPage.svelte`

```svelte
<script>
  import ContextMenu from "@lgn/frontend/src/components/ContextMenu.svelte";

  import myContextMenu from "../actions/contextMenu";
  import myContextMenuStore from "../stores/contextMenu";

  // By default our context menu entry do nothing
  // and immediately close the context menu.

  contextMenuStore.register("my-context-menu", [
    {
      type: "item",
      label: "Do this",
      onClick({ close, payload }) {
        console.log(payload); // Will print "I am a payload"
        close();
      },
    },
    { type: "separator" },
    {
      type:
      "item",
      label: "Do that",
      onClick({ close }) { close(); },
    },
  ]);

  contextMenuStore.register("my-other-context-menu", [
    {
      type: "item",
      label: "Do this other thing",
      onClick({ close }) { close(); },
    },
    { type: "separator" },
    {
      type: "item",
      label: "Do that other thing",
      onClick({ close }) { close(); },
    },
  ]);
</script>

<ContextMenu contextMenuStore={myContextMenuStore}

<div>
  <div>If you right click me, a default context menu is shown.</div>
  <div use:myContextMenu={{
    name: "my-context-menu",
    payload: "I am a payload",
  }}>
    If you right click me "Do this" and "Do that" entries are displayed.
  </div>
  <div use:myContextMenu={{ name: "my-other-context-menu" }}>
    If you right click me "Do this other thing"
    and "Do that other thing" entries are displayed.
  </div>
</div>
```
-->
<script lang="ts">
  import { Readable } from "svelte/store";

  import clickOutside from "../actions/clickOutside";
  import { remToPx } from "../lib/html";
  import log from "../lib/log";
  import { sleep } from "../lib/promises";
  import { Position } from "../lib/types";
  import { Entry, StoreValue } from "../stores/contextMenu";

  type State =
    | { type: "hidden" }
    | { type: "appearing"; position: Position }
    | { type: "disappearing"; position: Position }
    | { type: "shown"; position: Position };

  const entryHeightRem = 2.5;

  const entryHeightPx = remToPx(entryHeightRem) as number;

  const separatorHeightPx = 1;

  const marginPx = 2;

  const widthRem = 20;

  const widthPx = remToPx(widthRem) as number;

  const defaultEntries: Entry[] = [
    {
      type: "item",
      label: "Help",
      onClick({ close }) {
        close();
      },
    },
    {
      type: "item",
      label: "About",
      onClick({ close }) {
        close();
      },
    },
  ];

  export let contextMenuStore: Readable<StoreValue<string>>;

  let state: State = { type: "hidden" };

  let currentEntries: Entry[] = [];

  let currentPayload: unknown;

  function computePositionFrom(
    { clientX, clientY, view }: MouseEvent,
    entries: Entry[]
  ): Position {
    // Should not happen
    if (!view) {
      throw new Error("`window` object not attached to `event`");
    }

    const entriesNb = entries.filter((entry) => entry.type === "item").length;

    const separatorsNb = entries.filter(
      (entry) => entry.type === "separator"
    ).length;

    const heightPx =
      entriesNb * entryHeightPx + separatorsNb * separatorHeightPx;

    const x =
      view.innerWidth - clientX <= widthPx
        ? clientX - (widthPx - (view.innerWidth - clientX)) - marginPx
        : clientX + marginPx;

    const y =
      view.innerHeight - clientY <= heightPx
        ? view.innerHeight - heightPx - (view.innerHeight - clientY) - marginPx
        : clientY + marginPx;

    return { x, y };
  }

  async function handleContextMenu(event: MouseEvent) {
    // In dev mode `Ctrl + Right Click` will open the default
    // context menu for dev purpose.
    if (import.meta.env.DEV && event.ctrlKey) {
      return;
    }

    event.preventDefault();
    event.stopPropagation();

    // Recursively get the menu name, if any
    const contextMenuData = (function rec(
      element: HTMLElement
    ): { name: string; payload?: unknown } | null {
      if (element.dataset.contextMenu) {
        return {
          name: element.dataset.contextMenu,
          payload: element.dataset.contextMenuPayload,
        };
      }

      if (!element.parentElement) {
        return null;
      }

      return rec(element.parentElement);
    })(event.target as HTMLElement);

    const newCurrentEntries =
      (contextMenuData &&
        $contextMenuStore &&
        $contextMenuStore[contextMenuData.name]) ||
      defaultEntries;

    let position = computePositionFrom(event, newCurrentEntries);

    // First we "close" the current context menu if it's open
    if ("position" in state) {
      state = { type: "disappearing", position: state.position };

      await sleep(50).promise;
    }

    currentEntries = newCurrentEntries;

    currentPayload =
      (contextMenuData &&
        contextMenuData.payload &&
        typeof contextMenuData.payload === "string" &&
        JSON.parse(contextMenuData.payload)) ||
      null;

    state = {
      type: "appearing",
      position,
    };

    await sleep(50).promise;

    state = {
      type: "shown",
      position: state.position,
    };
  }

  async function close() {
    if (!("position" in state)) {
      return;
    }

    state = { type: "disappearing", position: state.position };

    await sleep(50).promise;

    state = { type: "hidden" };
  }
</script>

<svelte:window on:contextmenu={handleContextMenu} />

<div
  class="root"
  class:opacity-100={state.type === "shown"}
  class:opacity-0={state.type === "disappearing" || state.type === "appearing"}
  class:block={state.type === "appearing" ||
    state.type === "shown" ||
    state.type === "disappearing"}
  class:hidden={state.type === "hidden"}
  use:clickOutside={close}
  style={"position" in state
    ? `width: ${widthRem}rem; top: ${state.position.y}px; left: ${state.position.x}px;`
    : `width: ${widthRem}rem;`}
>
  <div class="entries">
    {#each currentEntries as entry, index (index)}
      {#if entry.type === "item"}
        <div
          class="item"
          class:danger={entry.tag === "danger"}
          on:click={() => entry.onClick({ close, payload: currentPayload })}
        >
          {entry.label}
        </div>
      {:else if entry.type === "separator"}
        <div class="separator" />
      {/if}
    {/each}
  </div>
</div>

<style lang="postcss">
  .root {
    @apply absolute bg-gray-800 z-50 rounded-sm transition-opacity ease-in duration-[50ms] shadow-xl;
  }

  .entries {
    @apply flex flex-col py-1 text-lg;
  }

  .item {
    @apply flex items-center px-4 h-10 cursor-pointer hover:bg-gray-500;
  }

  .item.danger {
    @apply text-red-500;
  }

  .separator {
    @apply h-px bg-gray-400;
  }
</style>
