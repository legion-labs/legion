<template>
  <v-app-bar id="app-bar" dark dense clipped-left app>
    <v-app-bar-nav-icon @click="localDrawer = !drawer"></v-app-bar-nav-icon>
    <v-app-bar-title>Legion Labs Editor</v-app-bar-title>
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
      downCoordinates: null,
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

    // Handle both double-click and drag of the window.
    appBar.onmousedown = (e) => {
      this.downCoordinates = { x: e.clientX, y: e.clientY };

      e.preventDefault();
    };

    appBar.onmouseup = (e) => {
      this.downCoordinates = null;
    };

    appBar.onmousemove = (e) => {
      if (this.downCoordinates) {
        if (
          Math.abs(e.clientX - this.downCoordinates.x) > 2 ||
          Math.abs(e.clientY - this.downCoordinates.y) > 2
        ) {
          this.startDragging();
        }
      }
    };

    appBar.ondblclick = (e) => {
      this.toggle();
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
    async restore() {
      await this.windowManager.unmaximize();
      this.maximized = false;
    },
    async toggle() {
      await this.windowManager.toggleMaximize();
      this.maximized = !this.maximized;
    },
    close() {
      this.windowManager.close();
    },
  },
};
</script>
