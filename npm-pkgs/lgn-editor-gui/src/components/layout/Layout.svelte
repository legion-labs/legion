<script lang="ts">
  import "golden-layout/dist/css/goldenlayout-base.css";
  import "golden-layout/dist/css/themes/goldenlayout-dark-theme.css";
  import { onMount } from "svelte";
  import {
    ComponentContainer,
    ResolvedComponentItemConfig,
    VirtualLayout,
  } from "golden-layout";
  import { layoutConfig, type EditorComponents } from "./LayoutConfig";

  type LayoutComponent = {
    container: ComponentContainer;
    visible: boolean;
    type: EditorComponents;
    zIndex: string;
    rect: {
      height: number;
      width: number;
      left: number;
      top: number;
    };
  };

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
    console.log(itemConfig);

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
        console.log(dz);
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
        type: ResolvedComponentItemConfig.resolveComponentTypeName(itemConfig),
        visible: true,
        container: container,
      } as LayoutComponent,
    ];

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
</script>

<div class="layout" bind:offsetHeight={height} bind:offsetWidth={width}>
  <div class="virtual-layout-container" use:initializeLayout />
  {#each layoutComponents as c (c.container)}
    <div
      class="component"
      class:hidden={!c.visible}
      style:z-index={c.zIndex}
      style:left={`${c.rect.left}px`}
      style:top={`${c.rect.top}px`}
      style:width={`${c.rect.width}px`}
      style:height={`${c.rect.height}px`}
    >
      <slot type={c.type} />
    </div>
  {/each}
</div>

<style>
  .layout {
    background-color: fuchsia;
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
    overflow: auto;
    position: absolute;
  }

  .component.hidden {
    display: none;
  }
</style>
