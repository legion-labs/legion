<template>
  <div class="hello">
    <h1>Legion Editor</h1>
    <video id="view"></video>
    <pre id="logs"></pre>
  </div>
</template>

<script>
import { invoke } from "@tauri-apps/api/tauri";
//import { VideoPlayer } from "@/video";

export default {
  name: "Video",
  props: {
    msg: String,
  },
  mounted() {
    const videoElement = document.getElementById("view");
    //const videoPlayer = new VideoPlayer(videoElement, () => {});
    const logsElement = document.getElementById("logs");

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

      video_channel.onopen = async () => {
        logsElement.append(
          document.createTextNode("Opened video data channel.\n")
        );
      };

      video_channel.onclose = async () => {
        logsElement.append(
          document.createTextNode("Closed video data channel.\n")
        );
      };

      video_channel.onmessage = async (msg) => {
        //videoPlayer.push(msg.data);
      };
    };
  },
};
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped>
video {
  border: 1px solid black;
  border-radius: 8px;
  background: url("~assets/v.png") center center no-repeat #222;
  min-width: 320px;
  min-height: 240px;
  cursor: pointer;
}

pre {
  border: 1px solid black;
  border-radius: 8px;
  min-width: 320px;
  min-height: 240px;
  background-color: #222;
  color: white;
}
</style>
