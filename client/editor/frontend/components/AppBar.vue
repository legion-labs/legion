<template>
  <v-app-bar dark dense clipped-left app>
    <v-app-bar-nav-icon @click="localDrawer = !drawer"></v-app-bar-nav-icon>
    <v-spacer></v-spacer>
    <v-btn icon>
      <v-avatar>
        <v-icon>mdi-account-circle</v-icon>
      </v-avatar>
    </v-btn>

    <v-btn icon :disabled="true"></v-btn>

    <v-btn icon @click="minimize()">
      <v-icon>mdi-window-minimize</v-icon>
    </v-btn>

    <v-btn v-if="maximized" icon @click="restore()">
      <v-icon>mdi-window-restore</v-icon>
    </v-btn>
    <v-btn v-else icon @click="maximize()">
      <v-icon>mdi-window-maximize</v-icon>
    </v-btn>

    <v-btn icon @click="close()">
      <v-icon>mdi-window-close</v-icon>
    </v-btn>
  </v-app-bar>
</template>

<script>
export default {
  name: "AppBar",
  props: {
    drawer: false,
  },
  data() {
    return {
      maximized: false,
    };
  },
  computed: {
    localDrawer: {
      get() {
        return this.drawer;
      },
      set(val) {
        this.$emit("drawer-change", val);
      },
    },
  },
  mounted() {
    this.windowManager = new window.__TAURI__.window.WindowManager();
    const appBar = this.$el;

    // Prevent the onmousedown from child buttons to trigger the window move
    // procedure.
    appBar.getElementsByTagName("button").forEach((element) => {
      element.onmousedown = (e) => {
        e.stopPropagation();
      };
    });

    appBar.onmousedown = (e) => {
      e.stopPropagation();
      e.preventDefault();

      this.startDragging();
    };
  },
  methods: {
    startDragging() {
      this.windowManager.startDragging();
    },
    minimize() {
      this.windowManager.minimize();
    },
    async maximize() {
      await this.windowManager.maximize();
      this.maximized = true;
    },
    restore() {
      this.windowManager.unmaximize();
      this.maximized = false;
    },
    close() {
      this.windowManager.close();
    },
  },
};
</script>
