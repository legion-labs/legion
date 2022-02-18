<script>
  import Header from "@/components/Misc/Header.svelte";
</script>

<div class="w-full h-screen p-2">
  <div class="grid">
    <Header />
    <div class="pl-5 pr-5 pb-5">
      <div id="app">
        <slot />
      </div>
    </div>
  </div>
</div>
