<script lang="ts">
  import resizeAction from "../actions/resize";
  import { nullable as nullableAction } from "../lib/action";
  import type { TabTypeBase, WorkspaceStore } from "../stores/workspace";

  type TabType = $$Generic<TabTypeBase>;

  const resize = nullableAction(resizeAction);

  export let id: string;

  export let workspace: WorkspaceStore<TabType>;

  $: tile = $workspace.tiles.find((tile) => tile.id === id);

  function onResize({ height, width }: DOMRectReadOnly) {
    if (!tile || tile.size.type === "untracked") {
      return;
    }

    tile.size.value = { height, width };
  }
</script>

{#if tile}
  <div class="root" use:resize={tile.size.type === "tracked" ? onResize : null}>
    <slot {tile} />
  </div>
{:else}
  <div>Unknown tile id {id}</div>
{/if}

<style lang="postcss">
  .root {
    @apply h-full w-full;
  }
</style>
