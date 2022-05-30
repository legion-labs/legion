<script lang="ts" context="module">
  export type ComponentLayoutState = {
    state: object;
    onClosed?: () => void;
  };
</script>

<script lang="ts">
  import {
    ComponentContainer,
    ComponentItemConfig,
    ResolvedComponentItemConfig,
    VirtualLayout,
  } from "golden-layout";
  import type { LayoutConfig } from "golden-layout";
  import "golden-layout/dist/css/goldenlayout-base.css";
  import "golden-layout/dist/css/themes/goldenlayout-dark-theme.css";
  import { onMount } from "svelte";
  import type { SvelteComponentDev } from "svelte/internal";

  type LayoutComponent = {
    type: string;
    id: string;
    layoutState: ComponentLayoutState;
    container: ComponentContainer;
    visible: boolean;
    zIndex: string;
    rect: {
      height: number;
      width: number;
      left: number;
      top: number;
    };
  };

  export let layoutConfig: LayoutConfig;
  export let componentMap: Record<string, typeof SvelteComponentDev>;
  export let surfaceClass: string | null;

  let height: number;
  let width: number;
  let layout: VirtualLayout;
  let layoutComponents: LayoutComponent[] = [];
  let layoutRect: DOMRect;

  onMount(() => {
    layout.loadLayout(layoutConfig);
  });

  $: if (width || height) {
    layout?.setSize(
      layout.container.offsetWidth,
      layout.container.offsetHeight
    );
  }

  export function addComponent(
    componentType: string,
    id: string,
    componentState?: ComponentLayoutState,
    title?: string
  ) {
    // Poor man's way to optionally ensure unicity per id which currently allows to avoid opening <SceneExplorer> duplicates.
    // A better system would check if a given component is allowed to be created multiple times (singleton or singleton-by-key).
    const lc = layoutComponents.find((l) => l.id === id);

    if (lc) {
      lc.container.focus();

      return;
    }

    const config: ComponentItemConfig = {
      type: "component",
      componentType: componentType,
      componentState,
      title,
      id,
    };

    const existingOfType = layoutComponents.find(
      (c) => c.type === componentType
    );

    if (existingOfType !== undefined) {
      const existingLayoutComponent = layout.findFirstComponentItemById(
        existingOfType.id
      );

      if (existingLayoutComponent !== undefined) {
        existingLayoutComponent.layoutManager.addItem(config);
      }
    } else {
      return layout.addItemAtLocation(config, [
        { typeId: 4, index: 0 },
        { typeId: 7, index: 1 },
      ]);
    }
  }

  function initializeLayout(divElement: HTMLDivElement) {
    layout = new VirtualLayout(
      divElement,
      onBindComponentEvent,
      onUnbindComponentEvent
    );

    layout.beforeVirtualRectingEvent = (_) => {
      layoutRect = divElement.getBoundingClientRect();
    };

    return {
      destroy() {
        layout.destroy();
      },
    };
  }

  function getComponentByContainerReference(c: ComponentContainer) {
    return layoutComponents.find((lc) => lc.container === c);
  }

  function refreshComponents() {
    layoutComponents = layoutComponents;
  }

  function onBindComponentEvent(
    container: ComponentContainer,
    itemConfig: ResolvedComponentItemConfig
  ) {
    container.virtualRectingRequiredEvent = (c, width, height) => {
      const layoutComponent = getComponentByContainerReference(c);

      if (layoutComponent) {
        const rect = c.element.getBoundingClientRect();
        const left = rect.left - layoutRect.left;
        const top = rect.top - layoutRect.top;

        layoutComponent.rect = { left, top, width, height };
        refreshComponents();
      }
    };

    container.virtualZIndexChangeRequiredEvent = (c, _, dz) => {
      const layoutComponent = getComponentByContainerReference(c);

      if (layoutComponent) {
        layoutComponent.zIndex = dz;
        refreshComponents();
      }
    };

    container.virtualVisibilityChangeRequiredEvent = (c, v) => {
      const layoutComponent = getComponentByContainerReference(c);

      if (layoutComponent) {
        layoutComponent.visible = v;
        refreshComponents();
      }
    };

    layoutComponents = [
      ...layoutComponents,
      {
        rect: {},
        id: itemConfig.id,
        layoutState: itemConfig.componentState,
        type: ResolvedComponentItemConfig.resolveComponentTypeName(itemConfig),
        visible: true,
        container,
      } as LayoutComponent,
    ];

    const state = itemConfig.componentState as ComponentLayoutState;

    container.on("initialised", () => {
      if (state.onClosed) {
        container.tab.closeElement?.addEventListener(
          "click",
          () => {
            if (state.onClosed) {
              state.onClosed();
            }
          },
          { once: true }
        );
      }
    });

    return {
      virtual: true,
      component: undefined,
    };
  }

  function onUnbindComponentEvent(container: ComponentContainer) {
    const layoutComponent = getComponentByContainerReference(container);

    if (layoutComponent) {
      layoutComponents.splice(layoutComponents.indexOf(layoutComponent), 1);
      refreshComponents();
    }
  }

  function initialised(_: HTMLElement, c: LayoutComponent) {
    c.container.emit("initialised");
  }
</script>

<div class="layout" bind:offsetHeight={height} bind:offsetWidth={width}>
  <div class="virtual-layout-container" use:initializeLayout />
  {#each layoutComponents as c (c.container)}
    <div
      use:initialised={c}
      class="component"
      class:bg-surface={surfaceClass}
      class:hidden={!c.visible}
      style:z-index={c.zIndex}
      style:left={`${c.rect.left}px`}
      style:top={`${c.rect.top}px`}
      style:width={`${c.rect.width}px`}
      style:height={`${c.rect.height}px`}
    >
      <svelte:component this={componentMap[c.type]} {...c.layoutState.state} />
    </div>
  {/each}
</div>

<style lang="postcss">
  .layout {
    position: relative;
    width: 100%;
    height: 100%;
    overflow: hidden;
  }

  .virtual-layout-container {
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    top: 0;
  }

  .component {
    @apply absolute;
    overflow: visible;
  }

  .component.hidden {
    display: none;
  }
</style>
