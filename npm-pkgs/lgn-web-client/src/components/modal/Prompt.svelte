<script lang="ts">
  import Button from "../Button.svelte";
  import Modal from "./Modal.svelte";

  export let id: symbol;

  export let title = "Confirmation";

  export let message = "Are you sure?";

  export let cancelLabel = "Cancel";

  export let confirmLabel = "Yes";

  export let close: () => void;

  function sendAnswer(answer: boolean) {
    window.dispatchEvent(
      new CustomEvent("prompt-answer", { detail: { answer, id } })
    );

    close();
  }
</script>

<Modal on:close={close}>
  <div slot="title">
    <div>{title}</div>
  </div>
  <div class="body" slot="body">
    {message}
  </div>
  <div class="footer" slot="footer">
    <div class="buttons">
      <div>
        <Button size="lg" on:click={() => sendAnswer(false)}>
          {cancelLabel}
        </Button>
      </div>
      <div>
        <Button variant="danger" size="lg" on:click={() => sendAnswer(true)}>
          {confirmLabel}
        </Button>
      </div>
    </div>
  </div>
</Modal>

<style lang="postcss">
  .body {
    @apply flex flex-col px-2 py-4;
  }

  .footer {
    @apply flex flex-row justify-end w-full;
  }

  .footer .buttons {
    @apply flex flex-row space-x-2;
  }
</style>
