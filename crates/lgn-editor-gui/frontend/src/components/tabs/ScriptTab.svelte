<script lang="ts">
  import { createEventDispatcher } from "svelte";

  import ScriptEditor from "@lgn/web-client/src/components/ScriptEditor.svelte";

  import type { ScriptTabTypePayload } from "@/stores/tabPayloads";

  const dispatch = createEventDispatcher<{ change: ScriptTabTypePayload }>();

  export let payload: ScriptTabTypePayload;

  export let payloadId: string;
</script>

{#key payloadId}
  <ScriptEditor
    theme="vs-dark"
    on:change={({ detail: newValue }) =>
      dispatch("change", { ...payload, value: newValue })}
    value={payload.value}
    readonly={payload.readonly}
    lang={payload.lang}
  />
{/key}
