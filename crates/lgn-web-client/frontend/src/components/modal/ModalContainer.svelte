<!--
@Component

_Not to be mistaken with the `Modal` component._

The modal container is meant to be mounted once, at the root, in the page/application.

Once mounted it will listen to the `open-modal` custom event (which you can easily dispatch
using the `openModal` function provided by the `lib/modal.ts` module) and open up accordingly.
-->
<script lang="ts">
  import { fade } from "svelte/transition";
  import ModalStore from "../../stores/modal";

  export let store: ModalStore;

  $: ids = Object.getOwnPropertySymbols($store);
</script>

{#each ids as id (id)}
  {@const { content, config } = $store[id]}

  <div
    class="root"
    class:with-lg-margin={window.__TAURI_METADATA__}
    on:keydown={(event) => event.key === "Escape" && store.close(id)}
    transition:fade={{ duration: config?.noTransition ? 0 : 100 }}
    tabindex={-1}
  >
    <div>
      <svelte:component this={content} close={() => store.close(id)} {config} />
    </div>
  </div>
{/each}

<style lang="postcss">
  .root {
    @apply flex justify-center items-center bg-black bg-opacity-90 absolute inset-0 w-screen z-30 mt-8 h-[calc(100vh-theme("spacing.10"))];
  }

  .root.with-lg-margin {
    @apply mt-10;
  }
</style>
