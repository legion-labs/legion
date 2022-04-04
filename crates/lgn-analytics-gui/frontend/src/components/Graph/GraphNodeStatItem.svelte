<script lang="ts">
  import { formatExecutionTime } from "@/lib/format";

  import { GraphNodeStatType } from "./Lib/GraphNodeStatType";

  export let type: GraphNodeStatType;
  export let value: number;

  type StatTypeDesc = {
    icon?: string;
    format: (v: number) => string;
  };

  const statTypeUnit: Record<GraphNodeStatType, StatTypeDesc> = {
    [GraphNodeStatType.Max]: {
      icon: "bi bi-chevron-bar-right",
      format: (v) => formatExecutionTime(v),
    },
    [GraphNodeStatType.Min]: {
      icon: "bi bi-chevron-bar-left",
      format: (v) => formatExecutionTime(v),
    },
    [GraphNodeStatType.Sum]: {
      icon: "bi bi-caret-right-fill",
      format: (v) => formatExecutionTime(v),
    },
    [GraphNodeStatType.Avg]: {
      icon: "bi bi-chevron-bar-contract",
      format: (v) => formatExecutionTime(v),
    },
    [GraphNodeStatType.Count]: {
      icon: "bi bi-caret-right",
      format: (v) => v.toLocaleString(),
    },
  };
</script>

<div>
  <i class={statTypeUnit[type].icon} />
  <span class="font-semibold">
    {GraphNodeStatType[type]}
  </span>
  {statTypeUnit[type].format(value)}
</div>
