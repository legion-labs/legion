<script lang="ts">
  import Modal from "@lgn/frontend/src/components/modal/Modal.svelte";
  import Button from "@lgn/frontend/src/components/Button.svelte";
  import { sleep } from "@lgn/frontend/src/lib/promises";
  import { AsyncStoreOrchestrator } from "@lgn/frontend/src/stores/asyncStore";
  import Select from "../inputs/Select.svelte";
  import TextInput from "../inputs/TextInput.svelte";

  const requestStore = new AsyncStoreOrchestrator();

  const { loading } = requestStore;

  // TODO: Fetch types on the server side
  const typeOptions = [
    { id: "foo", value: "foo", label: "Foo" },
    { id: "bar", value: "bar", label: "Bar" },
    { id: "baz", value: "baz", label: "Baz" },
    { id: "qux", value: "qux", label: "Qux" },
    { id: "quux", value: "quux", label: "Quux" },
    { id: "qux2", value: "qux2", label: "Qux2" },
    { id: "foofoo", value: "foofoo", label: "Foofoo" },
    { id: "nnnn", value: "nnnn", label: "Nnnn" },
    { id: "truc", value: "truc", label: "Truc" },
  ];

  let name = "";

  let type: typeof typeOptions[number] | "" = "";

  async function createResource(event: SubmitEvent) {
    event.preventDefault();

    // Simulate a long request
    await requestStore.run(() => sleep(1_000));
  }
</script>

<form on:submit={createResource}>
  <Modal>
    <div slot="title">
      <div>Create New Resource</div>
    </div>
    <div class="body" slot="body">
      <label>
        <div class="field">
          <div>Name</div>
          <div>
            <TextInput
              bind:value={name}
              autoFocus
              disabled={$loading}
              size="lg"
            />
          </div>
        </div>
      </label>
      <label>
        <div class="field">
          <div>Type</div>
          <div>
            <Select
              options={typeOptions}
              size="lg"
              disabled={$loading}
              bind:value={type}
            >
              <div slot="option" let:option>{option.label}</div>
            </Select>
          </div>
        </div>
      </label>
    </div>
    <div class="footer" slot="footer" let:close>
      <div class="buttons">
        <div>
          <Button size="lg" on:click={close} disabled={$loading}>Cancel</Button>
        </div>
        <div>
          <Button variant="success" size="lg" type="submit" disabled={$loading}>
            Create
          </Button>
        </div>
      </div>
    </div>
  </Modal>
</form>

<style lang="postcss">
  .body {
    @apply flex flex-col space-y-4 px-2 py-4;

    .field {
      @apply flex flex-col space-y-1;
    }
  }

  .footer {
    @apply flex flex-row justify-end w-full;

    .buttons {
      @apply flex flex-row space-x-2;
    }
  }
</style>
