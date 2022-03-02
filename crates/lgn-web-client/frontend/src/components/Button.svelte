<script lang="ts">
  export let type: "submit" | "button" | "reset" = "button";

  export let variant: "notice" | "warning" | "success" | "danger" = "notice";

  export let size: "default" | "lg" = "default";

  export let disabled = false;

  /** Basically an `width: 100%` style so that the parent can control the width */
  export let fluid = false;
</script>

<button
  class="button"
  class:disabled
  class:w-full={fluid}
  class:notice={variant === "notice" && !disabled}
  class:warning={variant === "warning" && !disabled}
  class:success={variant === "success" && !disabled}
  class:danger={variant === "danger" && !disabled}
  class:default={size === "default" && !disabled}
  class:lg={size === "lg"}
  on:click
  {disabled}
  {type}><slot /></button
>

<style lang="postcss">
  .button {
    @apply flex justify-center h-full w-full px-4 outline-none items-center rounded-sm cursor-pointer transition-all duration-150 font-semibold shadow-sm shadow-gray-700 active:shadow-none;
  }

  .button.disabled {
    @apply bg-gray-700 cursor-not-allowed shadow-none;
  }

  .button.disabled :global(*) {
    @apply cursor-not-allowed;
  }

  .button.notice {
    @apply bg-gray-800 hover:bg-opacity-50;
  }

  .button.warning {
    @apply bg-orange-700 hover:bg-orange-800 text-gray-800 hover:text-white;
  }

  .button.success {
    @apply bg-green-700 hover:bg-green-800;
  }

  .button.danger {
    @apply bg-red-600 hover:bg-red-800 text-white;
  }

  .button.default {
    @apply h-8 text-lg;
  }

  .button.lg {
    @apply h-10 text-xl;
  }
</style>
