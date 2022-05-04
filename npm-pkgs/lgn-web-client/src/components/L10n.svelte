<script lang="ts">
  import type { FluentVariable } from "@fluent/bundle";
  import { getContext } from "svelte";

  import { l10nOrchestratorContextKey } from "../constants";
  import type { L10nOrchestrator } from "../orchestrators/l10n";

  export let customL10nOrchestrator: L10nOrchestrator | undefined = undefined;

  export let id: string;

  export let args: Record<string, FluentVariable> | null = null;

  const l10n =
    customL10nOrchestrator ||
    getContext<L10nOrchestrator | undefined>(l10nOrchestratorContextKey);

  if (!l10n) {
    throw new Error(
      "Unable to access the l10n orchestrator, you need to whether pass it down using \
the `customL10nOrchestrator` prop or set one in a context with the `l10nOrchestratorContextKey` \
constant exported by the `L10n` component"
    );
  }

  const { t } = l10n;
</script>

{$t(id, args)}
