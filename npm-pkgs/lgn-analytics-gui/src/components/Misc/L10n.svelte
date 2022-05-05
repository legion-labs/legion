<script lang="ts">
  import L10n from "@lgn/web-client/src/components/L10n.svelte";
  import type { ResolveFluentRecordVariablesOnly } from "@lgn/web-client/src/types/fluent";

  type Id = $$Generic<keyof Fluent>;

  type $$Props = ResolveFluentRecordVariablesOnly<Fluent, Id>;
</script>

<L10n id={$$props.id} {...$$props} />
