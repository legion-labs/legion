<script lang="ts">
    import { performanceAnalyticsClient } from "@/lib/performanceAnalyticsClient";
    import {
        MetricDesc,
        ProcessMetricReply,
    } from "@lgn/proto-telemetry/dist/analytics";
    import { onDestroy, onMount } from "svelte";
    import uPlot from "uplot";
    import "uplot/dist/uPlot.min.css";

    export let processId: string;
    let loading = true;
    let selectedMetrics: MetricDesc[];
    let minMs: number;
    let maxMs: number;
    let div: HTMLElement;
    let uplot: uPlot | undefined;
    const resizeObserver = new ResizeObserver((e) => {
        uplot?.setSize({
            width: div.clientWidth,
            height: uplot.height,
        });
    });

    onDestroy(() => {
        resizeObserver.disconnect();
    });

    onMount(async () => {
        resizeObserver.observe(div);

        const availableMetrics =
            await performanceAnalyticsClient.list_process_metrics({
                processId,
            });

        selectedMetrics = availableMetrics.metrics;
        minMs = availableMetrics.minTimeMs;
        maxMs = availableMetrics.maxTimeMs;

        const fetchedMetrics = await Promise.all(
            selectedMetrics.map((m) => {
                return performanceAnalyticsClient.fetch_process_metric({
                    processId,
                    metricName: m.name,
                    beginMs: minMs,
                    endMs: maxMs,
                });
            })
        );

        function buildChart(data: ProcessMetricReply[]) {
            const series = selectedMetrics.map((m, i) => {
                return {
                    label: `${m.name} (${m.unit})`,
                    stroke: ` hsla(${hashString(m.name) % 360}, 100%, 50%, 1)`,
                    scale: selectedMetrics[i].unit,
                } as uPlot.Series;
            });

            const axes = availableMetrics.metrics.map((m) => {
                return {
                    scale: m.unit,
                    side: availableMetrics.metrics.indexOf(m) == 0 ? 1 : 3,
                    ticks: 10,
                    label: m.unit,
                    size: 100,
                    show: availableMetrics.metrics.indexOf(m) == 1,
                } as uPlot.Axis;
            });

            const opts = {
                title: "Metrics",
                width: window.innerWidth,
                height: 500,
                series: [{}, ...series],
                scales: {
                    x: {
                        time: false,
                    },
                },
                axes: [{}, ...axes],
            };

            const xAxisData = new Set(
                data
                    .flatMap((m) => m.points.map((p) => p.timeMs))
                    .sort((a, b) => a + b)
            );

            const yAxisData = data.map((m) => m.points.map((p) => p.value));

            const uPlotData = [
                Array.from(xAxisData),
                ...yAxisData,
            ] as uPlot.AlignedData;

            uplot = new uPlot(opts, uPlotData, div);
        }

        buildChart(fetchedMetrics);

        loading = false;
    });

    function hashString(string: String): number {
        var hash = 0;
        for (var i = 0; i < string.length; i++) {
            hash = string.charCodeAt(i) + ((hash << 5) - hash);
            hash = hash & hash;
        }
        return hash;
    }
</script>

<div>
    <div bind:this={div} />
</div>
{#if loading}
    <h1>Loading...</h1>
{/if}
