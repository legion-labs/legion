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
  ></NumberWidget>
  <StringWidget
    v-else-if="isStringType(ptype)"
    v-model="localValue"
    :readonly="readonly"
  ></StringWidget>
  <VecWidget
    v-else-if="isVecType(ptype)"
    v-model="localValue"
    :readonly="readonly"
  ></VecWidget>
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
      ptype = ptype.toLowerCase();
      return ["bool"].includes(ptype);
    },
    isNumberType(ptype) {
      ptype = ptype.toLowerCase();
      return ["i32", "u32", "f32", "f64"].includes(ptype);
    },
    isStringType(ptype) {
      ptype = ptype.toLowerCase();
      return ["string"].includes(ptype);
    },
    isVecType(ptype) {
      ptype = ptype.toLowerCase();
      return ["vec3", "quat"].includes(ptype);
    },
  },
};
</script>
