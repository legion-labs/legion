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
    direct
  ></SpeedWidget>
  <BooleanWidget
    v-else-if="isBooleanType(ptype)"
    v-model="localValue"
    :readonly="readonly"
  ></BooleanWidget>
  <NumberWidget
    v-else-if="isNumberType(ptype)"
    v-model="localValue"
    :readonly="readonly"
    direct
  ></NumberWidget>
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
  methods: {
    isBooleanType(ptype) {
      return ["bool"].includes(ptype);
    },
    isNumberType(ptype) {
      return ["u32"].includes(ptype);
    },
  },
};
</script>
