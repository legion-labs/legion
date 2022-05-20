<script lang="ts">
  import type { NotificationsStore } from "../stores/notifications";
  import type { FluentBase } from "../types/fluent";
  import L10n from "./L10n.svelte";

  export let store: NotificationsStore<FluentBase>;
</script>

<div class="root">
  {#each Object.getOwnPropertySymbols($store) as key}
    {@const notification = $store[key]}

    <div
      class="notification"
      on:click={() => store.close(key)}
      on:mouseenter={() => store.pause(key)}
      on:mouseleave={() => store.resume(key)}
    >
      <div
        class="title"
        class:success={notification.type === "success"}
        class:warning={notification.type === "warning"}
        class:error={notification.type === "error"}
      >
        {#if notification.payload.type === "raw"}
          {notification.payload.title}
        {:else}
          <L10n {...notification.payload.title} />
        {/if}
      </div>
      <div class="message">
        {#if notification.payload.type === "raw"}
          {notification.payload.message}
        {:else}
          <L10n {...notification.payload.message} />
        {/if}
      </div>
      <div class="progress">
        {#if typeof notification.percentage === "number"}
          <div
            class="progress-inner"
            style="width: {notification.percentage}%;"
          />
        {/if}
      </div>
    </div>
  {/each}
</div>

<style lang="postcss">
  .root {
    @apply absolute right-4 top-12 space-y-4 z-50;
  }

  .notification {
    @apply w-80 bg-gray-800 shadow-lg shadow-gray-800 rounded-sm hover:bg-gray-700 cursor-pointer;
  }

  .title {
    @apply border-b-2 px-2 py-2 break-words;
  }

  .title.success {
    @apply border-green-800;
  }

  .title.warning {
    @apply border-orange-800;
  }

  .title.error {
    @apply border-red-800;
  }

  .message {
    @apply px-2 py-2 break-words;
  }

  .progress {
    @apply h-2 bg-gray-700 rounded-b-sm;
  }

  .progress-inner {
    @apply h-full bg-gray-400 rounded-b-sm;
  }
</style>
