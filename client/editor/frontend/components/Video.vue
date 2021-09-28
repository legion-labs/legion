<template>
  <video id="video" fill-height fluid></video>
</template>

<script scoped>
import { invoke } from "@tauri-apps/api/tauri";

export default {
  name: "Video",
  props: {
    msg: String,
  },
  mounted() {
    const videoElement = document.getElementById("video");

    videoElement.onclick = function () {
      console.log("Initializing WebRTC...");

      const pc = new RTCPeerConnection({
        urls: [{ url: "stun:stun.l.google.com:19302" }],
      });

      pc.onnegotiationneeded = async () => {
        pc.setLocalDescription(await pc.createOffer());
      };

      pc.onicecandidate = async (iceEvent) => {
        console.log(iceEvent);

        if (iceEvent.candidate === null) {
          console.log(JSON.stringify(pc.localDescription.toJSON()));

          const rtcSessionDescription = await invoke("initialize_stream", {
            rtcSessionDescription: btoa(
              JSON.stringify(pc.localDescription.toJSON())
            ),
          });

          pc.setRemoteDescription(
            new RTCSessionDescription(JSON.parse(atob(rtcSessionDescription)))
          );
        }
      };

      const video_channel = pc.createDataChannel("video");

      video_channel.onopen = async () => {};

      video_channel.onclose = async () => {};

      video_channel.ontrack = async (event) => {
        videoElement.srcObject = event.streams[0];
        console.log("Video track: ", event.streams[0]);
      };
    };
  },
};
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped>
video {
  cursor: pointer;
  background: url("~assets/images/disconnected.png") center center no-repeat;
  background-color: black;
  background-size: 20%;
  width: 100%;
}
</style>
