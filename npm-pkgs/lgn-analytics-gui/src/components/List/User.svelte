<script lang="ts">
  export let user: string;
</script>

<div class="flex gap-2 items-center select-none">
  <div
    class="w-5 h-5 flex justify-center items-center rounded-full text-xs text-white-87 bg-orange-700"
  >
    <span class="capitalize">{user.charAt(0)}</span>
  </div>
  <span class="capitalize text-content-100">{user}</span>
</div>
