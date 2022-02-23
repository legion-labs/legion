<script lang="ts">
  import { addPropertyInPropertyVector as addPropertyInPropertyVectorApi } from "@/api";
  import { form as createForm, field } from "svelte-forms";
  import { required } from "svelte-forms/validators";
  import Modal from "@lgn/web-client/src/components/modal/Modal.svelte";
  import Button from "@lgn/web-client/src/components/Button.svelte";
  import { AsyncStoreOrchestrator } from "@lgn/web-client/src/stores/asyncStore";
  import Select from "../inputs/Select.svelte";
  import { getAvailableComponentTypes } from "@/api";
  import { GetAvailableDynTraitsResponse } from "@lgn/proto-editor/dist/property_inspector";
  import Field from "../Field.svelte";
  import { Config } from "@lgn/web-client/src/stores/modal";

  const createComponentStore = new AsyncStoreOrchestrator();

  const { loading } = createComponentStore;

  const dynTraitTypesStore =
    new AsyncStoreOrchestrator<GetAvailableDynTraitsResponse>();

  const type = field<{ value: string; item: string } | "">("type", "", [
    required(),
  ]);

  const createComponentForm = createForm(type);

  export let close: () => void;

  // We don't get any payload when the user tries to create
  // a resource at the top level
  export let config: Config<{
    id: string;
    path: string;
    index: number;
  }>;

  async function getComponentList(event: Event /* SubmitEvent */) {
    event.preventDefault();

    // Simulate a long request
    await createComponentStore.run(async () => {
      await createComponentForm.validate();

      if (!$createComponentForm.valid || !$type.value) {
        return;
      }

      if (config.payload) {
        const jsonValue = `{"${$type.value.item}": {}}`;

        const path = config.payload.path;

        const value = await addPropertyInPropertyVectorApi(config.payload.id, {
          path: config.payload.path,
          index: config.payload.index,
          jsonValue,
        });

        window.dispatchEvent(
          new CustomEvent("refresh-property", { detail: { path, value } })
        );
      }

      close();

      return $type.value;
    });
  }
</script>

<form on:submit={getComponentList}>
  <Modal on:close={close}>
    <div slot="title">
      <div>Create New Component</div>
    </div>
    <div class="body" slot="body">
      <Field field={type}>
        <div slot="label">Component Type</div>
        <div slot="input">
          {#await dynTraitTypesStore.run(getAvailableComponentTypes) then { availableTraits }}
            <Select
              bind:value={$type.value}
              options={availableTraits.map((traitType) => ({
                item: traitType,
                value: traitType,
              }))}
              size="lg"
              disabled={$loading}
              status={$type.invalid ? "error" : "default"}
            >
              <div slot="option" let:option>{option.item}</div>
            </Select>
          {:catch error}
            <div>
              Couldn't retrieve the component type from the server: {error}
            </div>
          {/await}
        </div>
        <div slot="error" let:error>
          Component type is {error}
        </div>
      </Field>
    </div>
    <div class="footer" slot="footer">
      <div class="buttons">
        <div>
          <Button size="lg" on:click={close} disabled={$loading}>Cancel</Button>
        </div>
        <div>
          <Button
            variant="success"
            size="lg"
            type="submit"
            disabled={$loading || !$type.value}
          >
            Create
          </Button>
        </div>
      </div>
    </div>
  </Modal>
</form>

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
