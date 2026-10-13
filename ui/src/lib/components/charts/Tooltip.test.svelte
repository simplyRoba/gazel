<script lang="ts">
  import { LayerCake, Svg } from "layercake";
  import { scaleLinear } from "d3-scale";

  import Tooltip from "./Tooltip.svelte";

  interface Point {
    x: number;
    y: number;
    date: string;
    value: number;
  }

  let {
    points = [
      { x: 20, y: 30, date: "First", value: 1 },
      { x: 80, y: 10, date: "Second", value: 2 },
    ],
    width = 100,
    height = 50,
  }: { points?: Point[]; width?: number; height?: number } = $props();
</script>

<LayerCake
  ssr
  data={points}
  x="x"
  y="y"
  xScale={scaleLinear()}
  xDomain={[0, 100]}
  yDomain={[0, 50]}
  yReverse={false}
  containerWidth={width}
  containerHeight={height}
>
  <Svg>
    <Tooltip
      formatX={(point: Record<string, unknown>) => String(point.date)}
      formatY={(point: Record<string, unknown>) => String(point.value)}
    />
  </Svg>
</LayerCake>
