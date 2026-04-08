<script lang="ts">
  let { size = 32 }: { size?: number } = $props();
</script>

<!--
  Gazelle mark — 7 triangles, no background.
  Uses currentColor so it adapts to light/dark theme via CSS.
  Original viewBox matches docs/assets/logo.svg geometry.
-->
<svg
  viewBox="90 55 280 360"
  width={size}
  height={size}
  fill="currentColor"
  aria-hidden="true"
>
  <!-- Horn 1 (back) -->
  <polygon points="202,69 289,133 300,131" opacity="0.7" />
  <!-- Horn 2 (front) -->
  <polygon points="208,67 294,121 302,118" opacity="0.9" />
  <!-- Head -->
  <polygon points="301,135 335,182 290,160" opacity="0.9" />
  <!-- Neck -->
  <polygon points="289,165 358,200 282,257" opacity="0.8" />
  <!-- Chest + front leg -->
  <polygon points="358,204 315,399 282,261" opacity="0.8" />
  <!-- Back + back leg -->
  <polygon points="276,262 143,248 171,400" opacity="0.6" />
  <!-- Tail -->
  <polygon points="106,206 138,243 155,246" opacity="0.6" />
</svg>
