<script lang="ts">
  import ChartCard from "./ChartCard.svelte";

  let {
    data,
    minimumDataPoints = 2,
  }: {
    data: { x: number; y: number }[];
    minimumDataPoints?: number;
  } = $props();
</script>

<ChartCard title="Test chart" {data} x="x" y="y" {minimumDataPoints}>
  <g data-testid="chart-content"></g>
</ChartCard>
