<template>
  <v-app>
    <v-app-bar dense clipped-left app>
      <v-app-bar-nav-icon @click="drawer = !drawer"></v-app-bar-nav-icon>
      <v-spacer></v-spacer>
      <v-btn icon>
        <v-avatar>
          <v-icon>mdi-account-circle</v-icon>
        </v-avatar>
      </v-btn>
    </v-app-bar>
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
    <v-footer app>
      <v-spacer></v-spacer>
      <span>&copy; {{ new Date().getFullYear() }} - Legion Labs</span>
      <v-spacer></v-spacer>
    </v-footer>
  </v-app>
</template>

<style>
html {
  overflow-y: auto;
}
</style>

<style scoped>
.v-footer {
  border-top: 1px solid silver;
  font-style: italic;
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
