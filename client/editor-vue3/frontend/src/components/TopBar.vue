<script lang="ts" setup>
import useClickOutside from "@/composables/useClickOutside";
import { Id as MenuId, menus, useTopBarMenu } from "@/stores/topBarMenu";

// Props and events
type Props = {
  documentTitle?: string;
};

defineProps<Props>();

// Reactive values

const topBarMenu = useTopBarMenu();

const { ref: menuSectionRef } = useClickOutside(() => {
  if (topBarMenu.openedMenuId) {
    topBarMenu.close();
  }
});

// Callbacks

const onMenuMouseEnter = (id: MenuId) =>
  // We set the openedMenu value (and therefore open said menu dropdown)
  // only when a menu is open
  topBarMenu.isOpen && topBarMenu.set(id);

const onMenuClick = (id: MenuId) => {
  // Simple menu dropdown display toggle
  topBarMenu.isOpen ? topBarMenu.close() : topBarMenu.set(id);
};

const onMenuItemClick = () => {
  // When a user clicks on a menu dropdown item, we just close the menu
  topBarMenu.close();
  console.log("Executed");
};
</script>

<template>
  <div class="flex flex-row justify-between space-x-2">
    <div
      ref="menuSectionRef"
      class="flex flex-row flex-1 h-7 space-x-1 text-sm"
    >
      <div class="flex items-center italic px-2">Legion</div>
      <div
        v-for="menu in menus"
        :key="menu.id"
        class="flex items-center hover:bg-gray-400 cursor-pointer"
        :class="{ 'bg-gray-400': topBarMenu.openedMenuId === menu.id }"
        @mouseenter="onMenuMouseEnter(menu.id)"
        @click="onMenuClick(menu.id)"
      >
        <div class="px-2">
          {{ menu.title }}
        </div>
        <div
          class="absolute top-7"
          :class="{ hidden: topBarMenu.openedMenuId !== menu.id }"
        >
          <div class="bg-gray-800 py-1 bg-opacity-90">
            <div
              v-for="menuItemTitle in [
                `Foo ${menu.title}`,
                `Bar ${menu.title}`,
                `Baz ${menu.title}`,
              ]"
              :key="menuItemTitle"
              class="cursor-pointer hover:bg-gray-400 px-6 py-0.5"
              @click.stop="onMenuItemClick"
            >
              {{ menuItemTitle }}
            </div>
          </div>
        </div>
      </div>
    </div>
    <div
      class="flex flex-row justify-center items-center flex-1 whitespace-nowrap"
    >
      <template v-if="documentTitle">{{ documentTitle }}</template>
      <template v-else>Untitled document</template>
    </div>
    <div class="flex-1" />
  </div>
</template>
