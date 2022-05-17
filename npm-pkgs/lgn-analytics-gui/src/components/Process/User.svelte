<script lang="ts">
  export let user: string;

  export let nameOnly = false;
</script>

<div class="flex gap-2 items-center select-none">
  <div
    class="w-5 h-5 flex justify-center items-center rounded-full text-xs text-white bg-orange-700"
  >
    <span class="capitalize">{user.charAt(0)}</span>
  </div>
  {#if !nameOnly}
    <div class="capitalize">
      <slot {user}>
        {user}
      </slot>
    </div>
  {/if}
</div>
