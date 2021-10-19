<template>
  <ColorWidget
    v-if="ptype == 'color'"
    v-model="localValue"
    :readonly="readonly"
  ></ColorWidget>
  <SpeedWidget
    v-else-if="ptype == 'speed'"
    v-model="localValue"
    :readonly="readonly"
  ></SpeedWidget>
  <JSONWidget v-else v-model="localValue" :readonly="readonly"></JSONWidget>
</template>

<script>
export default {
  name: "ResourcePropertyEditor",
  props: {
    value: {},
    readonly: {
      type: Boolean,
      default: false,
    },
    ptype: String,
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
