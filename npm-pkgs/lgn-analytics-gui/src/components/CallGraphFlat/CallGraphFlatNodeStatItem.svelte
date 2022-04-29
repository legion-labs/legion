<script lang="ts">
  import { CallGraphNodeStatType } from "@/lib/CallGraph/CallGraphNodeStatType";
  import { formatExecutionTime } from "@/lib/format";

  export let type: CallGraphNodeStatType;
  export let value: number;

  type StatTypeDesc = {
    icon?: string;
    format: (v: number) => string;
  };

  const statTypeUnit: Record<CallGraphNodeStatType, StatTypeDesc> = {
    [CallGraphNodeStatType.Max]: {
      icon: "bi bi-chevron-bar-right",
      format: (v) => formatExecutionTime(v),
    },
    [CallGraphNodeStatType.Min]: {
      icon: "bi bi-chevron-bar-left",
      format: (v) => formatExecutionTime(v),
    },
    [CallGraphNodeStatType.Sum]: {
      icon: "bi bi-caret-right-fill",
      format: (v) => formatExecutionTime(v),
    },
    [CallGraphNodeStatType.Avg]: {
      icon: "bi bi-chevron-bar-contract",
      format: (v) => formatExecutionTime(v),
    },
    [CallGraphNodeStatType.Count]: {
      icon: "bi bi-caret-right",
      format: (v) => v.toLocaleString(),
    },
    [CallGraphNodeStatType.Sd]: {
      icon: "bi bi-lightbulb",
      format: (v) => formatExecutionTime(v),
    },
  };
</script>

<div>
  <i class={statTypeUnit[type].icon} />
  <span class="font-semibold">
    {CallGraphNodeStatType[type]}
  </span>
  {statTypeUnit[type].format(value)}
</div>
