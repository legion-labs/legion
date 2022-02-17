<script lang="ts">
  export let user: string;

  function hashString(string: string): number {
    let hash = 0;
    for (let i = 0; i < string.length; i++) {
      hash = string.charCodeAt(i) + ((hash << 5) - hash);
      hash = hash & hash;
    }
    return hash;
  }
</script>

<div class="flex gap-2 items-center">
  <div
    class="w-6 h-6 flex justify-center items-center rounded-full text-xs text-white"
    style="background:hsl({hashString(user) % 360}, 65%, 50%)"
  >
    <span class="capitalize">{user.charAt(0)}</span>
  </div>
  <span class="capitalize">{user}</span>
</div>
