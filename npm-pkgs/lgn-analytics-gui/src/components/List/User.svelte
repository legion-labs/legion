<script lang="ts">
  export let user: string;
</script>

<div class="flex gap-2 items-center select-none">
  <div
    class="w-6 h-6 flex justify-center items-center rounded-full text-xs text-white-87 bg-orange-700"
  >
    <span class="capitalize">{user.charAt(0)}</span>
  </div>
  <span class="user">{user}</span>
</div>



<style lang="postcss">
.user {
 @apply font-default capitalize text-sm; 
}
</style>