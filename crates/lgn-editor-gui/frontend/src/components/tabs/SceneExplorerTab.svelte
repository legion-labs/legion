<script lang="ts">
  import { allActiveScenes } from "@/orchestrators/allActiveScenes";
  import type { SceneExplorerTypePayload } from "@/stores/tabPayloads";

  import SceneExplorer from "../SceneExplorer.svelte";

  export let payloadId: string;

  export let payload: SceneExplorerTypePayload;

  $: activeScene = $allActiveScenes
    ? $allActiveScenes.find(
        ({ rootScene }) => rootScene.id === payload.rootSceneId
      )
    : null;
</script>

{#key payloadId}
  {#if activeScene}
    <SceneExplorer
      rootScene={activeScene.rootScene}
      activeScenes={activeScene.scenes}
    />
  {/if}
{/key}
