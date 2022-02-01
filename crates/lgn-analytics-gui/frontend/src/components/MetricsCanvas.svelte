<script lang="ts">
	import { client } from "@/lib/client";
	import {
		MetricDesc,
		ProcessMetricReply,
	} from "@lgn/proto-telemetry/dist/analytics";
	import * as d3 from "d3";
	import { onMount } from "svelte";
	export let id: string;

	interface Point {
		time: number;
		value: number;
	}

	const margin = { top: 20, right: 50, bottom: 60, left: 70 };

	const outerHeight = 600;
	const height = outerHeight - margin.top - margin.bottom;

	let mainWidth: number = 0;
	$: width = mainWidth - margin.left - margin.right;

	let totalMinMs = -Infinity;
	let totalMaxMs = Infinity;
	let metricsDesc: MetricDesc[] = [];
	let metrics: ProcessMetricReply[] = [];
	let points: Point[][] = [];
	let loading = true;
	let updateTime: number;

	let x: d3.ScaleTime<number, number, never>;
	let y: d3.ScaleLinear<number, number, never>;

	let gxAxis: d3.Selection<SVGGElement, unknown, HTMLElement, any>;
	let gyAxis: d3.Selection<SVGGElement, unknown, HTMLElement, any>;
	let xAxis: d3.Axis<d3.NumberValue>;
	let yAxis: d3.Axis<d3.NumberValue>;

	let container: d3.Selection<d3.BaseType, unknown, HTMLElement, any>;
	let context: CanvasRenderingContext2D;
	let transform: d3.ZoomTransform = d3.zoomIdentity;
	let canvas: HTMLCanvasElement;

	$: {
		if (mainWidth && container && transform) {
			updateChart();
		}
	}

	onMount(async () => {
		await fetchDataAsync().then(() => (loading = false));
		createChart();
		updateChart();
	});

	async function fetchDataAsync() {
		const reply = await client.list_process_metrics({ processId: id });
		metricsDesc = reply.metrics;
		totalMinMs = reply.minTimeMs;
		totalMaxMs = reply.maxTimeMs;
		metrics = await Promise.all(
			metricsDesc.map((m) => {
				return client.fetch_process_metric({
					processId: id,
					metricName: m.name,
					beginMs: totalMinMs,
					endMs: totalMaxMs,
				});
			})
		);

		points = metrics.map((m) =>
			m.points.map((p) => {
				return <Point>{
					time: p.timeMs,
					value: p.value,
				};
			})
		);
	}

	function createChart() {
		container = d3.select(".canvas");

		const svgGroup = container
			.append("svg")
			.append("g")
			.attr("transform", `translate(${margin.left}, ${margin.top})`);

		const canvasChart = container
			.append("canvas")
			.style("position", "absolute")
			.style("top", 0)
			.style("left", 0)
			.style("margin-left", `${margin.left}px`)
			.style("margin-top", `${margin.top}px`);

		canvas = canvasChart.node() as HTMLCanvasElement;

		context = canvas.getContext("2d")!;

		x = d3.scaleTime().domain([totalMinMs, totalMaxMs]).nice();
		y = d3.scaleLinear().nice();

		xAxis = d3.axisBottom(x);
		yAxis = d3.axisLeft(y);

		gxAxis = svgGroup
			.append("g")
			.attr("transform", `translate(0, ${height})`)
			.call(xAxis);

		gyAxis = svgGroup.append("g").call(yAxis);

		const zoom = d3
			.zoom()
			.scaleExtent([1, 1000])
			.translateExtent([[0, 0], getTranslateExtent()])
			.on("zoom", (event) => {
				transform = event.transform;
			});

		container.call(zoom as any);
	}

	function updateChart() {
		var startTime = performance.now();

		container
			.select("svg")
			.attr("height", outerHeight)
			.attr("width", width);

		container
			.select("canvas")
			.attr("height", height)
			.attr("width", width - margin.left);

		x.range([0, width]);

		const yMax = d3.max(
			points.flatMap((p) => d3.max(p.map((p) => p.value)) ?? 0)
		);

		y.range([height, 0]).domain([0, yMax ?? 0]);

		draw();

		updateTime = Math.floor(performance.now() - startTime);
	}

	function draw() {
		const scaleX = transform.rescaleX(x);

		context.fillStyle = "rgba(0, 0, 0, 0)";
		context.fillRect(0, 0, width, height);

		var line = d3
			.line()
			.x((d) => scaleX(d[0]))
			.y((d) => y(d[1]))
			.context(context);

		points.forEach((points, i) => {
			context.beginPath();
			line(points.map((p) => [p.time, p.value]));
			context.strokeStyle = d3.schemeCategory10[i];
			context.lineWidth = 0.33;
			context.stroke();
		});

		gxAxis.call(xAxis.scale(scaleX));
		gyAxis.call(yAxis.scale(y));
	}

	function getTranslateExtent(): [number, number] {
		return [mainWidth, outerHeight];
	}
</script>

<div bind:clientWidth={mainWidth}>
	<div class="canvas" />
	{#if loading}
		<div>Loading...</div>
	{:else}
		<div>
			<div>Width: {width}</div>
			<div>Main Width {mainWidth}</div>
			<div>updateTime: {updateTime} ms</div>
			<div>
				Transform: X{Math.floor(transform.x)} Y{Math.floor(transform.y)}
				K{transform.k}
			</div>
			<ul>
				<li>
					<span class="font-bold">Min</span>
					{totalMinMs.toFixed(2)}
				</li>
				<li>
					<span class="font-bold">Max</span>
					{totalMaxMs.toFixed(2)}
				</li>
			</ul>
			<br />
			<ul>
				{#each metricsDesc as md}
					<li>
						{md.name} (unit: {md.unit})
					</li>
				{/each}
			</ul>
			<div>
				{#each metrics as metric, index}
					<div>
						[{index}] {metric.points.length} points
					</div>
				{/each}
			</div>
		</div>
	{/if}
</div>
