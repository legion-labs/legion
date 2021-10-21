<template>
  <span v-if="readonly">{{ localValue }}</span>
  <v-edit-dialog
    large
    persistent
    dark
    save-text="Apply"
    :return-value.sync="localValue"
    v-else
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
    value: Number,
    readonly: {
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