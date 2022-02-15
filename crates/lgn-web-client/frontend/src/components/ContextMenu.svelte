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
  import { fade } from "svelte/transition";
  import clickOutside from "../actions/clickOutside";
  import { remToPx } from "../lib/html";
  import { sleep } from "../lib/promises";
  import { Position } from "../lib/types";
  import ContextMenuStore from "../stores/contextMenu";
  import { buildCustomEvent, Entry, ItemEntry } from "../types/contextMenu";

  const entryHeightRem = 2.5;

  const entryHeightPx = remToPx(entryHeightRem) as number;

  const separatorHeightPx = 1;

  const marginPx = 2;

  const widthRem = 20;

  const widthPx = remToPx(widthRem) as number;

  const fadeDuration = 100;

  const defaultEntries: Entry[] = [
    { action: "help", type: "item", label: "Help" },
    { action: "about", type: "item", label: "About" },
  ];

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  export let store: ContextMenuStore<Record<string, Entry[]>>;

  // Position is null when the context menu is closed
  let position: Position | null = null;

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
    // In dev mode `Ctrl + Right Click` will open the default
    // context menu for dev purpose.
    if (import.meta.env.DEV && event.ctrlKey) {
      return;
    }

    event.preventDefault();
    event.stopPropagation();

    // Do not do anything when right clicking inside a context menu element
    if (
      position &&
      contextMenu &&
      event.target instanceof Node &&
      contextMenu.contains(event.target)
    ) {
      return;
    }

    if (position) {
      close();

      await sleep(fadeDuration + 16);
    }

    currentEntries = defaultEntries;

    entrySetName = null;

    position = computePositionFrom(event, defaultEntries);
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

    if (position) {
      close();

      await sleep(fadeDuration + 16);
    }

    currentEntries = ($store && $store[event.detail.name]) || defaultEntries;

    entrySetName = event.detail.name;

    position = computePositionFrom(event.detail.originalEvent, currentEntries);
  }

  function close() {
    position = null;
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

{#if position}
  <div
    class="root"
    style={position
      ? `width: ${widthRem}rem; top: ${position.y}px; left: ${position.x}px;`
      : `width: ${widthRem}rem;`}
    bind:this={contextMenu}
    on:click-outside={(event) => {
      event.detail.originalEvent.button !== 2 && close();
    }}
    use:clickOutside={contextMenu && [contextMenu]}
    transition:fade={{ duration: fadeDuration }}
  >
    <div class="entries">
      {#each currentEntries as entry, index (index)}
        {#if entry.type === "item"}
          <div
            class="item"
            class:danger={entry.tag === "danger"}
            on:click={() => dispatchContextMenuActionEvent(entry)}
          >
            {entry.label}
          </div>
        {:else if entry.type === "separator"}
          <div class="separator" />
        {/if}
      {/each}
    </div>
  </div>
{/if}

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
