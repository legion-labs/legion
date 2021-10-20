<template>
  <v-app>
    <AppBar :drawer="drawer" @drawer-change="drawer = $event"></AppBar>
    <v-navigation-drawer
      v-model="drawer"
      class="pt-4"
      color="grey lighten-3"
      mini-variant
      clipped
      app
    >
      <v-list>
        <v-list-item
          v-for="(item, i) in items"
          :key="i"
          :to="item.to"
          router
          exact
        >
          <v-list-item-action>
            <v-icon>{{ item.icon }}</v-icon>
          </v-list-item-action>
          <v-list-item-content>
            <v-list-item-title v-text="item.title" />
          </v-list-item-content>
        </v-list-item>
      </v-list>
    </v-navigation-drawer>
    <v-main>
      <v-container fill-height fluid class="pa-0 align-stretch">
        <Nuxt />
      </v-container>
    </v-main>
  </v-app>
</template>

<style>
html {
  overflow-y: auto;
}
</style>

<style scoped>
div.v-application {
  height: 100vh;
}

main {
  height: calc(100% - 1px);
  border-right: 1px solid #333;
  border-left: 1px solid #333;
  border-bottom: 1px solid #333;
}
</style>

<script>
export default {
  data() {
    return {
      drawer: true,
      items: [
        {
          icon: "mdi-apps",
          title: "Editor",
          to: "/",
        },
        {
          icon: "mdi-chart-bubble",
          title: "About",
          to: "/about",
        },
      ],
    };
  },
};
</script>
