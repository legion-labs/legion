<script lang="ts">
  export let type: "submit" | "button" | "reset" = "button";

  export let variant: "notice" | "warning" | "success" | "danger" = "notice";

  export let size: "default" | "lg" = "default";

  export let disabled = false;

  /** Basically an `width: 100%` style so that the parent can control the width */
  export let fluid = false;
</script>

<div
  class="root"
  class:disabled
  class:notice={variant === "notice" && !disabled}
  class:warning={variant === "warning" && !disabled}
  class:success={variant === "success" && !disabled}
  class:danger={variant === "danger" && !disabled}
  class:default={size === "default" && !disabled}
  class:lg={size === "lg"}
  class:w-full={fluid}
>
  <button class="button" on:click {disabled} {type}><slot /></button>
</div>

<style lang="postcss">
  .root {
    @apply flex justify-center items-center rounded-sm cursor-pointer transition-colors font-semibold;
  }

  .root.disabled {
    @apply bg-gray-700 cursor-not-allowed;
  }

  .root.disabled :global(*) {
    @apply cursor-not-allowed;
  }

  .root.notice {
    @apply bg-gray-800 hover:bg-opacity-50;
  }

  .root.warning {
    @apply bg-orange-700 hover:bg-orange-800 text-gray-800 hover:text-white;
  }

  .root.success {
    @apply bg-green-700 hover:bg-green-800;
  }

  .root.danger {
    @apply bg-red-600 hover:bg-red-800 text-gray-800;
  }

  .root.default {
    @apply h-8 text-lg;
  }

  .root.lg {
    @apply h-10 text-xl;
  }

  .button {
    @apply w-full h-full px-4 outline-none;
  }
</style>
