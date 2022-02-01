<!--
@Component

_Not to be mistaken with the `Modal` component._

The modal container is meant to be mounted once, at the root, in the page/application.

Once mounted it will listen to the `open-modal` custom event (which you can easily dispatch
using the `openModal` function provided by the `lib/modal.ts` module) and open up accordingly.
-->
<script lang="ts">
  import { SvelteComponent } from "svelte";
  import { OpenModalEvent } from "../../lib/modal";

  let content: SvelteComponent | null = null;

  function open({ detail }: OpenModalEvent) {
    content = detail.content;
  }

  function close() {
    content = null;
  }
</script>

<svelte:window on:open-modal={open} on:close-modal={close} />

{#if content}
  <div class="root" class:with-margin={window.__TAURI__}>
    <div>
      <svelte:component this={content} />
    </div>
  </div>
{/if}

<style lang="postcss">
  .root {
    @apply flex justify-center items-center bg-black bg-opacity-90 absolute inset-0 h-screen w-screen z-30;
  }

  .root.with-margin {
    @apply mt-10 h-[calc(100vh-theme("spacing.10"))];
  }
</style>
