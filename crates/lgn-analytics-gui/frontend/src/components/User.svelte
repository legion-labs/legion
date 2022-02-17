<script lang="ts">
  export let user: string;
</script>

<div class="flex gap-2 items-center">
  <div
    class="w-6 h-6 flex justify-center items-center rounded-full bg-gray-500 text-xs text-white"
  >
    <span class="capitalize">{user.charAt(0)}</span>
  </div>
  <span class="capitalize">{user}</span>
</div>
