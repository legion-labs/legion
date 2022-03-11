<script lang="ts">
  import type { createNotificationsStore } from "../stores/notifications";

  export let store: ReturnType<typeof createNotificationsStore>;
</script>

<div class="root">
  {#each Object.getOwnPropertySymbols($store) as key}
    {@const notification = $store[key]}

    <div class="notification" on:click={() => notification.close()}>
      <div
        class="title"
        class:success={notification.type === "success"}
        class:warning={notification.type === "warning"}
        class:error={notification.type === "error"}
      >
        {notification.title}
      </div>
      <div class="message">
        {notification.message}
      </div>
      <div class="progress">
        <div
          class="progress-inner"
          style="width: {notification.percentage}%;"
        />
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
    @apply border-b-2 px-2 py-2;
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
    @apply px-2 py-2;
  }

  .progress {
    @apply h-2 bg-gray-700 rounded-b-sm;
  }

  .progress-inner {
    @apply h-full bg-gray-400 rounded-b-sm;
  }
</style>
