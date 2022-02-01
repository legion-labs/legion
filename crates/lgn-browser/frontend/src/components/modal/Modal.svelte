<!--
@Component

_Not to be mistaken with the `Modal` component._

Simple Modal component wrapper. You can provided a title, a body, and a footer.

Some helpers, like `close`, will be passed down to the slots.
-->
<script lang="ts">
  import Icon from "@iconify/svelte";
  import { closeModal } from "../../lib/modal";
</script>

<div class="root">
  <div class="header">
    <div><slot name="title" close={closeModal} /></div>
    <div class="close" on:click={closeModal} title="Close modal">
      <Icon icon="ic:baseline-close" />
    </div>
  </div>
  <div class="body">
    <slot name="body" close={closeModal} />
  </div>
  <div class="footer">
    <slot name="footer" close={closeModal} />
  </div>
</div>

<style lang="postcss">
  .root {
    @apply w-96 bg-gray-700 rounded-sm shadow-lg shadow-black;

    .header {
      @apply flex flex-row justify-between items-center border-b-2 border-orange-700 px-2 py-1 text-lg h-12 font-semibold;

      .close {
        @apply flex justify-center items-center cursor-pointer text-orange-700;
      }
    }

    .body {
      @apply shadow-lg shadow-gray-800;
    }

    .footer {
      @apply bg-gray-500 rounded-b-sm px-2 py-1;
    }
  }
</style>
