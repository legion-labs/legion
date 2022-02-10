<script lang="ts">
  import { form as createForm, field } from "svelte-forms";
  import { required } from "svelte-forms/validators";
  import Modal from "@lgn/web-client/src/components/modal/Modal.svelte";
  import Button from "@lgn/web-client/src/components/Button.svelte";
  import { AsyncStoreOrchestrator } from "@lgn/web-client/src/stores/asyncStore";
  import Select from "../inputs/Select.svelte";
  import TextInput from "../inputs/TextInput.svelte";
  import {
    getResourceTypes,
    createResource as createResourceApi,
    getAllResources,
  } from "@/api";
  import {
    GetResourceTypeNamesResponse,
    ResourceDescription,
  } from "@lgn/proto-editor/dist/resource_browser";
  import allResourcesStore from "@/stores/allResources";
  import { join } from "@/lib/path";
  import Field from "../Field.svelte";

  const createResourceStore = new AsyncStoreOrchestrator();

  const { loading } = createResourceStore;

  const resourceTypesStore =
    new AsyncStoreOrchestrator<GetResourceTypeNamesResponse>();

  const name = field("name", "", [required()]);

  const type = field<{ value: string; item: string } | "">("type", "", [
    required(),
  ]);

  const createResourceForm = createForm(name, type);

  export let close: () => void;

  export let payload: ResourceDescription;

  async function createResource(event: SubmitEvent) {
    event.preventDefault();

    // Simulate a long request
    await createResourceStore.run(async () => {
      await createResourceForm.validate();

      if (!$createResourceForm.valid || !$type.value) {
        return;
      }

      // TODO: As soon as the folder-ish resources are supported, uncomment
      // const path = join([payload.path, $name.value]);
      const resourcePath = $name.value;

      const resourceType = $type.value.item;

      try {
        await createResourceStore.run(() =>
          createResourceApi({
            resourcePath,
            resourceType,
          })
        );
      } catch (error) {}

      close();

      allResourcesStore.run(getAllResources);
    });
  }
</script>

<form on:submit={createResource}>
  <Modal on:close={close}>
    <div slot="title">
      <div>Create New Resource</div>
    </div>
    <div class="body" slot="body">
      <div>
        <Field field={name}>
          <div slot="label">Resource Name</div>
          <div slot="input">
            <TextInput
              bind:value={$name.value}
              autoFocus
              disabled={$loading}
              size="lg"
              status={$name.invalid ? "error" : "default"}
            />
          </div>
          <div slot="error" let:error>
            Resource name is {error}
          </div>
        </Field>
      </div>
      <div>
        <Field field={type}>
          <div slot="label">Resource Type</div>
          <div slot="input">
            {#await resourceTypesStore.run(getResourceTypes) then { resourceTypes }}
              <Select
                bind:value={$type.value}
                options={resourceTypes.map((resourceType) => ({
                  item: resourceType,
                  value: resourceType,
                }))}
                size="lg"
                disabled={$loading}
                status={$type.invalid ? "error" : "default"}
              >
                <div slot="option" let:option>{option.item}</div>
              </Select>
            {:catch}
              <div>Couldn't retrieve the resource type from the server</div>
            {/await}
          </div>
          <div slot="error" let:error>
            Resource type is {error}
          </div>
        </Field>
      </div>
    </div>
    <div class="footer" slot="footer">
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
  }

  .footer {
    @apply flex flex-row justify-end w-full;

    .buttons {
      @apply flex flex-row space-x-2;
    }
  }
</style>
