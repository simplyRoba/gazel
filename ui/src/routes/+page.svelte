<script lang="ts">
  import { LayoutDashboard } from "lucide-svelte";
  import PageContainer from "$lib/components/PageContainer.svelte";
  import EmptyState from "$lib/components/EmptyState.svelte";
</script>

<PageContainer width="wide">
  <EmptyState
    icon={LayoutDashboard}
    heading="Welcome to gazel"
    description="Add your first vehicle in Settings to start tracking fill-ups and fuel efficiency."
  />
</PageContainer>
