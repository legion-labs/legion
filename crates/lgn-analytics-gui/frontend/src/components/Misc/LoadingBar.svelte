<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import ProgressBar from "progressbar.js";
  import { loadingStore } from "@/lib/Misc/LoadingStore";
  import { Unsubscriber } from "svelte/store";
  let subscription: Unsubscriber;
  let ratio: number;
  onMount(() => {
    loadingStore.reset();
    const line = new ProgressBar.Line("#loading-bar", {
      color: "#fc4d0f",
      svgStyle: {
        width: "100%",
        height: "100%",
        display: "block",
      },
    });
    subscription = loadingStore.subscribe(async (s) => {
      ratio = Math.pow(s.completed / s.requested, 10);
      line.animate(ratio, {
        duration: 300,
        easing: "easeInOut",
      });
    });
  });

  onDestroy(() => {
    if (subscription) {
      subscription();
    }
  });
</script>

<div id="loading-bar" style={`display:${ratio >= 1 ? "none" : "block"}`} />

<style>
  #loading-bar {
    position: fixed;
    top: 0;
    left: 0;
    height: 4px;
    width: 100%;
  }
</style>
