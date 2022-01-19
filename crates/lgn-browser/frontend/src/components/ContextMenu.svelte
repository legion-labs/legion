<!--
@Component
Context menu that replaces the default context menu in browsers and Tauri.

_This component is expected to be mounted _once_ in the whole page._

The component alone doesn't do much apart from replacing the context menu
by a new, custom, one, the so called "entries" are to be provided.

In order to add more entries to the context menu, one must first use
the `contextMenuStore` to register the new entries, and then, using
the `contextMenu` action let this component know what custom menu
the component must display.

Alongside the `type` and `label`, an `onClick` attribute must be provided.
This function will be called with the `close` function that allows
to close the context menu programmatically, and a `payload` (that can be
`undefined` if needed) to the `contextMenu` action.

While the `ContextMenu` component itself should be mounted only once,
the `contextMenuStore.register` can be called as many times as needed,
and a `contextMenuStore.remove` function is also provided to cleanup
unnecessary context menu entries.

## Example

`./stores/myContextMenu.ts`

```typescript
import buildContextMenuStore from "@lgn/frontend/src/stores/contextMenu";

// We define our context menu entry record with a simple type:
// keys represent the name of the entry set and values the payloads.
export type ContextMenuEntryRecord = {
  "my-context-menu": undefined;
  "my-other-context-menu": string | null;
};

export default buildContextMenuStore<ContextMenuEntryRecord>();
```

`./actions/myContextMenu.ts`

```typescript
import buildContextMenu from "@lgn/frontend/src/actions/contextMenu";
import myContextMenuStore, { ContextMenuEntryRecord } from "../stores/myContextMenu";

export default buildContextMenu<ContextMenuEntryRecord>(myContextMenuStore);
```

`./pages/MyPage.svelte`

```svelte
<script>
  import ContextMenu from "@lgn/frontend/src/components/ContextMenu.svelte";

  import myContextMenu from "../actions/myContextMenu";
  import myContextMenuStore from "../stores/myContextMenu";

  // `onClick` doesn't do anything and closes the context menu
  // right away. You can perform any kind of action in here
  // (including asynchronous ones).

  contextMenuStore.register("my-context-menu", [
    {
      type: "item",
      label: "Do this",
      onClick({ close }) { close(); },
    },
    { type: "separator" },
    {
      type:
      "item",
      label: "Do that",
      onClick({ close }) { close(); },
    },
  ]);

  // Since we properly defined our context entry record in our store,
  // the `payload` variable has type `string | null` below:

  contextMenuStore.register("my-other-context-menu", [
    {
      type: "item",
      label: "Do this other thing",
      onClick({ close, payload }) {
        console.log(payload); // Will print `"I am a payload"`

        close();
      },
    },
    { type: "separator" },
    {
      type: "item",
      label: "Do that other thing",
      onClick({ close, payload }) {
        console.log(payload); // Will print `"I am a payload"` too

        close(); },
    },
  ]);
</script>

<ContextMenu contextMenuStore={myContextMenuStore}

<div>
  <div>If you right click me, a default context menu is shown.</div>
  <div use:myContextMenu={{ name: "my-context-menu" }}>
    If you right click me "Do this" and "Do that" entries are displayed.
  </div>
  <div use:myContextMenu={{
    name: "my-other-context-menu",
    payload: "I am a payload",
  }}>
    If you right click me "Do this other thing"
    and "Do that other thing" entries are displayed.
  </div>
</div>
```
-->
<script lang="ts">
  import clickOutside from "../actions/clickOutside";
  import { remToPx } from "../lib/html";
  import { sleep } from "../lib/promises";
  import { Position } from "../lib/types";
  import { Entry, Store as ContextMenuStore } from "../stores/contextMenu";

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

  const defaultEntries: Entry<unknown>[] = [
    { type: "item", label: "Help", onClick: ({ close }) => close() },
    { type: "item", label: "About", onClick: ({ close }) => close() },
  ];

  export let contextMenuStore: ContextMenuStore<any>;

  $: entryRecord = contextMenuStore.entryRecord;

  $: activeEntrySet = contextMenuStore.activeEntrySet;

  let state: State = { type: "hidden" };

  let currentEntries: Entry<unknown>[] = [];

  let currentPayload: unknown;

  function computePositionFrom(
    { clientX, clientY, view }: MouseEvent,
    entries: Entry<unknown>[]
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

    const newCurrentEntries =
      ($activeEntrySet && $entryRecord && $entryRecord[$activeEntrySet.name]) ||
      defaultEntries;

    let position = computePositionFrom(event, newCurrentEntries);

    // First we "close" the current context menu if it's open
    if ("position" in state) {
      await close();
    }

    currentEntries = newCurrentEntries;

    currentPayload = $activeEntrySet?.payload ?? null;

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

    contextMenuStore.removeActiveEntrySet();

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
