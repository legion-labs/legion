<script lang="ts">
  import Modal from "@lgn/web-client/src/components/modal/Modal.svelte";
  import Button from "@lgn/web-client/src/components/Button.svelte";

  export let authorizationUrl: string;

  function redirect() {
    window.location.href = authorizationUrl;
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
