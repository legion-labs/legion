<script lang="ts">
  import { getContext } from "svelte";

  import { l10nOrchestratorContextKey } from "../constants";
  import type { L10nOrchestrator } from "../orchestrators/l10n";
  import type {
    FluentBase,
    ResolveFluentRecordVariablesOnly,
  } from "../types/fluent";

  type Fluent = $$Generic<FluentBase>;

  type Id = $$Generic<keyof Fluent>;

  type $$Props = ResolveFluentRecordVariablesOnly<Fluent, Id>;

  export let customL10nOrchestrator: L10nOrchestrator<Fluent> | undefined =
    undefined;

  const l10n =
    customL10nOrchestrator ||
    getContext<L10nOrchestrator<Fluent> | undefined>(
      l10nOrchestratorContextKey
    );

  if (!l10n) {
    throw new Error(
      "Unable to access the l10n orchestrator, you need to whether pass it down using \
the `customL10nOrchestrator` prop or set one in a context with the `l10nOrchestratorContextKey` \
constant exported by the `L10n` component"
    );
  }

  const { t } = l10n;
</script>

{$t($$props.id, $$props.variables)}
