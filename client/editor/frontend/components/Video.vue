<template>
  <div class="video-content d-flex">
    <video id="video"></video>
  </div>
</template>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped>
.video-content {
  cursor: pointer;
  position: relative;
  height: 100%;
  background: url("~assets/images/disconnected.png") center center no-repeat;
  background-color: black;
  background-size: 20%;
  background: linear-gradient(
    180deg,
    rgba(48, 48, 48, 1) 0%,
    rgba(0, 0, 0, 1) 100%
  );
}

video {
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  left: 0;
  right: 0;
  bottom: 0;
  width: auto;
  height: auto;
  margin-left: auto;
  margin-right: auto;
  max-width: 100%;
  max-height: 100%;
  object-fit: fill;
}
</style>

<script scoped>
import {
  initialize_stream,
  on_receive_control_message,
  on_send_edition_command,
} from "~/modules/api";

import { VideoPlayer } from "~/modules/video";
import { debounce, retryForever } from "~/modules/futures";

export default {
  name: "Video",
  props: {
    resource: null,
  },
  data() {
    return {
      pc: null,
      video_channel: null,
      control_channel: null,
    };
  },
  mounted() {
    const videoElement = document.getElementById("video");

    this.initialize(videoElement);
  },
  methods: {
    initialize(videoElement) {
      const videoPlayer = new VideoPlayer(videoElement, () => {});

      console.log("Initializing WebRTC...");

      if (this.video_channel != null) {
        this.video_channel.close();
        this.video_channel = null;
      }

      if (this.control_channel != null) {
        this.control_channel.close();
        this.control_channel = null;
      }

      if (this.pc !== null) {
        this.pc.close();
        this.pc = null;
      }

      this.pc = new RTCPeerConnection({
        urls: [{ url: "stun:stun.l.google.com:19302" }],
      });

      this.pc.onnegotiationneeded = async () => {
        this.pc.setLocalDescription(await this.pc.createOffer());
      };

      this.pc.onicecandidate = async (iceEvent) => {
        console.log(iceEvent);

        if (iceEvent.candidate === null) {
          retryForever(
            initialize_stream.bind(null, this.pc.localDescription)
          ).then((remoteDescription) =>
            this.pc.setRemoteDescription(remoteDescription)
          );
        }
      };

      this.pc.oniceconnectionstatechange = async () => {
        if (this.pc.iceConnectionState == "disconnected") {
          console.log("Disconnected");

          videoElement.pause();
          videoElement.removeAttribute("src");
          videoElement.load();

          window.setTimeout(() => {
            this.initialize(videoElement);
          }, 0);
        }
      };

      this.video_channel = this.pc.createDataChannel("video");
      this.control_channel = this.pc.createDataChannel("control");

      const observer = new ResizeObserver(
        debounce(async () => {
          console.log(
            "Sending resize event (",
            videoElement.parentElement.offsetWidth,
            videoElement.parentElement.offsetHeight,
            ")."
          );

          this.video_channel.send(
            JSON.stringify({
              event: "resize",
              width: videoElement.parentElement.offsetWidth,
              height: videoElement.parentElement.offsetHeight,
            })
          );
        }, 250)
      );

      this.video_channel.onerror = async (error) => {
        console.log(error.error);
      };
      this.video_channel.onopen = async () => {
        console.log("Video channel is now open.");
        observer.observe(videoElement.parentElement);
      };
      this.video_channel.onclose = async () => {
        console.log("Video channel is now closed.");
        observer.disconnect();
      };
      this.video_channel.onmessage = async (msg) => {
        videoPlayer.push(msg.data);
      };
      this.video_channel.ondatachannel = async (evt) => {
        console.log("video data channel: ", evt);
      };
      this.control_channel.onopen = async (evt) => {
        console.log("Control channel is now open: ", evt);
      };
      this.control_channel.onclose = async (evt) => {
        console.log("Control channel is now closed: ", evt);
      };
      this.control_channel.ondatachannel = async (evt) => {
        console.log("control data channel: ", evt);
      };
      this.control_channel.onmessage = async (msg) => {
        const json_msg = new TextDecoder().decode(msg.data);
        on_receive_control_message(json_msg);
      };
    },
  },
  watch: {
    resource(resource) {
      if (resource.description.id != "triangle") return;
      if (this.video_channel == null) return;

      for (const property of resource.properties) {
        var edition_event = null;

        if (property.name == "color") {
          edition_event = JSON.stringify({
            id: crypto.randomUUID(),
            event: "color",
            color: property.value,
          });
        } else if (property.name == "speed") {
          edition_event = JSON.stringify({
            id: crypto.randomUUID(),
            event: "speed",
            speed: property.value,
          });
        }

        if (edition_event) {
          on_send_edition_command(edition_event);
          this.video_channel.send(edition_event);
        }
      }
    },
  },
};
</script>
