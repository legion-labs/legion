<template>
  <pre v-if="readonly">{{ localValue }}</pre>
  <SpeedEditor v-else-if="direct" v-model="localValue"></SpeedEditor>
  <v-edit-dialog
    v-else
    large
    persistent
    dark
    save-text="Apply"
    :return-value.sync="localValue"
  >
    <pre>{{ localValue }}</pre>
    <template #input>
      <SpeedEditor v-model="localValue"></SpeedEditor>
    </template>
  </v-edit-dialog>
</template>

<script>
export default {
  name: "SpeedWidget",
  props: {
    // eslint-disable-next-line vue/require-default-prop
    value: Number,
    readonly: {
      type: Boolean,
      default: false,
    },
    direct: {
      type: Boolean,
      default: false,
    },
  },
  computed: {
    localValue: {
      get() {
        return this.value;
      },
      set(val) {
        this.$emit("input", val);
      },
    },
  },
};
</script>
