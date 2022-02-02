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

While the `ContextMenu` component itself should be mounted only once,
the `contextMenuStore.register` can be called as many times as needed,
and a `contextMenuStore.remove` function is also provided to cleanup
unnecessary context menu entries.

## Example

`./stores/myContextMenu.ts`

```typescript
import buildContextMenuStore from "@lgn/web-client/src/stores/contextMenu";

// We define our context menu entry record with a simple type:
// keys represent the name of the entry set
export type ContextMenuEntryRecord = {
  "my-context-menu": undefined;
  "my-other-context-menu": string | null;
};

export default buildContextMenuStore<ContextMenuEntryRecord>();
```

`./actions/myContextMenu.ts`

```typescript
import buildContextMenu from "@lgn/web-client/src/actions/contextMenu";
import { ContextMenuEntryRecord } from "../stores/myContextMenu";

export default buildContextMenu<ContextMenuEntryRecord>();
```

`./pages/MyPage.svelte`

```svelte
<script>
  import ContextMenu from "@lgn/web-client/src/components/ContextMenu.svelte";

  import myContextMenu from "../actions/myContextMenu";
  import myContextMenuStore, { ContextMenuEntryRecord } from "../stores/myContextMenu";

  contextMenuStore.register("my-context-menu", [
    {
      type: "item",
      action: "do-this",
      label: "Do this",
    },
    {
      type: "separator",
    },
    {
      type: "item",
      action: "do-that",
      label: "Do that",
    },
  ]);

  contextMenuStore.register("my-other-context-menu", [
    {
      type: "item",
      action: "do-something-else",
      label: "Do this other thing",
    },
    {
      type: "separator",
    },
    {
      type: "item",
      action: "do-another-thing",
      label: "Do that other thing",
    },
  ]);

  function handleContextMenuAction({
    detail: { action }
  }: ContextMenuActionEvent<ContextMenuEntryRecord>) {
    switch (action) {
      case "do-this": {
        // ...
      }

      // ...
    }
  }
</script>

<svelte:window on:contextmenu-action={handleContextMenuAction} />

<ContextMenu store={myContextMenuStore}

<div>
  <div>If you right click me, the default context menu is shown.</div>
  <div use:myContextMenu={"my-context-menu"}>
    If you right click me "Do this" and "Do that" entries are displayed.
  </div>
  <div use:myContextMenu={"my-other-context-menu"}>
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
  import ContextMenuStore from "../stores/contextMenu";
  import { buildCustomEvent, Entry, ItemEntry } from "../types/contextMenu";

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
    { action: "help", type: "item", label: "Help" },
    { action: "about", type: "item", label: "About" },
  ];

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  export let store: ContextMenuStore<any>;

  let state: State = { type: "hidden" };

  let currentEntries: Entry[] = [];

  let entrySetName: string | null = null;

  let contextMenu: HTMLElement | undefined;

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

  async function handleDefaultContextMenu(event: MouseEvent) {
    // Do not do anything when right clicking inside a context menu element
    if (
      state.type !== "hidden" &&
      contextMenu &&
      event.target instanceof Node &&
      contextMenu.contains(event.target)
    ) {
      event.preventDefault();
      event.stopPropagation();

      return;
    }

    // In dev mode `Ctrl + Right Click` will open the default
    // context menu for dev purpose.
    if (import.meta.env.DEV && event.ctrlKey) {
      return;
    }

    event.preventDefault();
    event.stopPropagation();

    let position = computePositionFrom(event, defaultEntries);

    // First we "close" the current context menu if it's open
    if ("position" in state) {
      await close();
    }

    currentEntries = defaultEntries;

    entrySetName = null;

    state = {
      type: "appearing",
      position,
    };

    await sleep(50);

    state = {
      type: "shown",
      position: state.position,
    };
  }

  async function handleCustomContextMenu(
    event: CustomEvent<{
      name: string;
      originalEvent: MouseEvent;
    }>
  ) {
    // In dev mode `Ctrl + Right Click` will open the default
    // context menu for dev purpose.
    if (import.meta.env.DEV && event.detail.originalEvent.ctrlKey) {
      return;
    }

    event.preventDefault();
    event.stopPropagation();

    const newCurrentEntries =
      ($store && $store[event.detail.name]) || defaultEntries;

    let position = computePositionFrom(
      event.detail.originalEvent,
      newCurrentEntries
    );

    // First we "close" the current context menu if it's open
    if ("position" in state) {
      await close();
    }

    currentEntries = newCurrentEntries;

    entrySetName = event.detail.name;

    state = {
      type: "appearing",
      position,
    };

    await sleep(50);

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

    await sleep(50);

    state = { type: "hidden" };
  }

  function dispatchContextMenuActionEvent(entry: ItemEntry) {
    if (entrySetName == null) {
      return;
    }

    window.dispatchEvent(buildCustomEvent(close, entrySetName, entry.action));
  }
</script>

<svelte:window
  on:contextmenu={handleDefaultContextMenu}
  on:custom-contextmenu={handleCustomContextMenu}
/>

<div
  class="root"
  class:opacity-100={state.type === "shown"}
  class:opacity-0={state.type === "disappearing" || state.type === "appearing"}
  class:block={state.type === "appearing" ||
    state.type === "shown" ||
    state.type === "disappearing"}
  class:hidden={state.type === "hidden"}
  style={"position" in state
    ? `width: ${widthRem}rem; top: ${state.position.y}px; left: ${state.position.x}px;`
    : `width: ${widthRem}rem;`}
  on:click-outside={close}
  use:clickOutside={contextMenu && [contextMenu]}
  bind:this={contextMenu}
>
  <div class="entries">
    {#each currentEntries as entry, index (index)}
      {#if entry.type === "item"}
        <div
          class="item"
          class:danger={entry.tag === "danger"}
          on:mouseup={() => dispatchContextMenuActionEvent(entry)}
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
    @apply absolute bg-gray-800 z-50 rounded-sm transition-opacity ease-in duration-[50ms] shadow-lg shadow-gray-800;
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
