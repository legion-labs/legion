<script lang="ts">
  let counter = 0;
</script>

<div>My About</div>

<div>Counter: {counter}</div>

<div on:click={() => (counter += 1)}>Increment</div>
