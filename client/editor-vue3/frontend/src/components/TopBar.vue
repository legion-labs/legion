<script lang="ts" setup>
import { Ref, ref } from "@vue/reactivity";
import useClickOutside from "@/composables/useClickOutside";

// Props and events
type Props = {
  documentTitle?: string;
};

defineProps<Props>();

// Types

// Obviously not meant to be used as is in production
// as the menu might become dynamic at one point
type Id = typeof menus[number]["id"];

// Values

const openedMenu: Ref<Id | null> = ref(null);

const menus = [
  { id: 1, title: "File" },
  { id: 2, title: "Edit" },
  { id: 3, title: "Layer" },
  { id: 4, title: "Document" },
  { id: 5, title: "View" },
  { id: 6, title: "Help" },
] as const;

const { ref: menuSectionRef } = useClickOutside(() => {
  if (openedMenu.value) {
    openedMenu.value = null;
  }
});

// Callbacks

const onMenuMouseEnter = (id: Id) =>
  // We set the openedMenu value (and therefore open said menu dropdown)
  // only when a menu is open
  openedMenu.value && (openedMenu.value = id);

const onMenuClick = (id: Id) => {
  // Simple menu dropdown display toggle
  openedMenu.value = openedMenu.value ? null : id;
};

const onMenuItemClick = () => {
  // When a user clicks on a menu dropdown item, we just close the menu
  openedMenu.value = null;
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
        :class="{ 'bg-gray-400': openedMenu === menu.id }"
        @mouseenter="onMenuMouseEnter(menu.id)"
        @click="onMenuClick(menu.id)"
      >
        <div class="px-2">
          {{ menu.title }}
        </div>
        <div class="absolute top-7" :class="{ hidden: openedMenu !== menu.id }">
          <div class="bg-gray-800 py-1 bg-opacity-90">
            <div
              v-for="menuItemTitle in [
                `Foo ${menu.title}`,
                `Bar ${menu.title}`,
                `Baz ${menu.title}`,
              ]"
              :key="menuItemTitle"
              class="cursor-pointer hover:bg-gray-400 px-6 py-0.5"
              @click="onMenuItemClick"
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
