<template>
  <span id="color">
    <span :style="bgColor" :tile="localValue"></span
    ><code>{{ localValue }}</code>
  </span>
</template>

<style scoped>
#color {
  height: 1.5em;
  line-height: 100%;
}

#color > * {
  height: inherit;
  display: inline-block;
  vertical-align: middle;
}

#color > span {
  width: 1.5em;
  border: 2px solid #333;
  background-color: var(--bg-color);
}

#color > code {
  cursor: text;
  border: 2px solid #333;
  border-left: 0;
  border-radius: 0;
}
</style>

<script>
import { u32_to_hexcolor, hexcolor_to_u32 } from "~/modules/conversion";

export default {
  name: "ColorViewer",
  props: ["value"],
  computed: {
    localValue: {
      get() {
        return u32_to_hexcolor(this.value);
      },
      set(val) {
        this.$emit("input", hexcolor_to_u32(val));
      },
    },
    bgColor() {
      return {
        "--bg-color": this.localValue,
      };
    },
  },
  mounted() {
    var self = this;

    document.querySelector("#color > code").onclick = function (e) {
      e.stopPropagation();
    };

    document.querySelector("#color > code").ondblclick = function (e) {
      e.stopPropagation();
      const selection = window.getSelection();
      const range = document.createRange();
      range.selectNodeContents(this);
      selection.removeAllRanges();
      selection.addRange(range);
    };
  },
};
</script>