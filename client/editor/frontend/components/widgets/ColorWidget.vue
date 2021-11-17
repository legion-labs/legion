<template>
  <ColorViewer v-if="readonly" v-model="localValue" readonly></ColorViewer>
  <v-edit-dialog
    v-else
    large
    persistent
    dark
    save-text="Apply"
    :return-value.sync="localValue"
  >
    <ColorViewer v-model="localValue"></ColorViewer>
    <template #input>
      <ColorEditor v-model="localValue"></ColorEditor>
    </template>
  </v-edit-dialog>
</template>

<script>
export default {
  name: "ColorWidget",
  props: {
    // eslint-disable-next-line vue/require-default-prop
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
