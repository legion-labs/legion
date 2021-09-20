<template>
  <div class="hello">
    <h1>Legion Editor</h1>
    <video></video>
  </div>
</template>

<script>
import { invoke } from "@tauri-apps/api/tauri";

invoke("initialize_stream", { rtcSessionDescription: "" })
  .then((res) => {
    console.log(
      "received RTC session description: ",
      res.rtcSessionDescription
    );
  })
  .catch((e) => {
    console.error("failed to initialize stream: ", e);
  });

export default {
  name: "HelloWorld",
  props: {
    msg: String,
  },
};
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped>
video {
  border: 1px solid black;
  border-radius: 8px;
  background: url("../assets/logo.png") center center no-repeat #222;
  min-width: 320px;
  min-height: 240px;
}
</style>
