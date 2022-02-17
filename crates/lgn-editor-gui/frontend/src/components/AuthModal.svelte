<script lang="ts">
  import Modal from "@lgn/web-client/src/components/modal/Modal.svelte";
  import Button from "@lgn/web-client/src/components/Button.svelte";
  import { Config } from "@lgn/web-client/src/stores/modal";

  // We don't get any payload when the user tries to create
  // a resource at the top level
  export let config: Config<string>;

  function redirect() {
    if (config.payload) {
      window.location.href = config.payload;
    }
  }
</script>

<Modal noClose>
  <div slot="title">
    <div>Authentication</div>
  </div>
  <div class="body" slot="body">
    <div>You must be logged in to continue.</div>
  </div>
  <div class="footer" slot="footer">
    <Button variant="success" size="lg" on:click={redirect}>Login</Button>
  </div>
</Modal>

<style lang="postcss">
  .body {
    @apply flex flex-col px-2 py-4;
  }

  .footer {
    @apply flex flex-row justify-end w-full;
  }
</style>
