<script lang="ts">
  import Panel from "@/components/Panel.svelte";
  import TopBar from "@/components/TopBar.svelte";
  import Video from "@/components/Video.svelte";
</script>

<div class="h-screen w-full">
  <TopBar />
  <div class="h-[calc(100vh-1.75rem)] w-full">
    <div class="flex flex-row h-full w-full">
      <div class="flex flex-col flex-shrink-0 w-3/4">
        <Panel>
          <span slot="header"> Untitled Document </span>
          <div class="h-full w-full" slot="content">
            <Video resource={null} />
          </div>
        </Panel>
      </div>
      <div class="flex-shrink-0 w-1" />
      <div class="flex flex-col w-full">
        <Panel>
          <span slot="header"> Properties </span>
        </Panel>
        <div class="flex-shrink-0 h-1" />
        <Panel>
          <span slot="header"> Layer Tree </span>
        </Panel>
      </div>
    </div>
  </div>
</div>
