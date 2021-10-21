<template>
  <div id="video-container" :style="backgroundURL()" class="d-flex">
    <video id="video" :class="{ show: !loading }"></video>
    <v-progress-linear
      id="loading"
      indeterminate
      color="yellow darken-2"
      v-if="loading"
    ></v-progress-linear>
    <code id="resolution" v-if="videoResolution" :class="{ show: !loading }">
      {{ videoResolution.width }}x{{ videoResolution.height }}</code
    >
    <h3 id="connecting" v-if="stateMsg">
      <v-progress-circular
        indeterminate
        color="white"
        :size="30"
      ></v-progress-circular>
      <span>{{ stateMsg }}</span>
    </h3>
  </div>
</template>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped>
#video-container {
  background-color: black;
  overflow: hidden;
  position: relative;
  background: linear-gradient(
      to top right,
      rgba(100, 115, 201, 0.7),
      rgba(25, 32, 72, 0.7)
    ),
    no-repeat center/cover var(--bg-url);
}

#video {
  position: absolute;
  object-fit: cover;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  max-width: 100%;
  max-height: 100%;
  margin: auto;
  opacity: 0;
  transition: opacity 0.5s linear;
}

#video.show {
  opacity: 1;
}

#loading {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
}

#resolution {
  position: absolute;
  border-radius: 1em;
  background-color: gray;
  color: black;
  top: 1em;
  right: 1em;
  opacity: 0;
  transition: opacity 0.5s linear;
  user-select: none;
  font-size: smaller;
}

#resolution.show {
  opacity: 0.5;
}

#connecting {
  position: absolute;
  left: 0;
  right: 0;
  top: 48%;
  text-align: center;
  color: silver;
  animation: glow 1s infinite alternate;
}

#connecting .v-progress-circular + span {
  margin-left: 1em;
}

@keyframes glow {
  from {
    opacity: 0.2;
  }

  to {
    opacity: 1;
  }
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
      videoResolution: null,
      desiredResolution: null,
      pc: null,
      video_channel: null,
      control_channel: null,
      stateMsg: null,
    };
  },
  computed: {
    loading() {
      return (
        !this.videoResolution ||
        !this.desiredResolution ||
        this.videoResolution.width != this.desiredResolution.width ||
        this.videoResolution.height != this.desiredResolution.height
      );
    },
  },
  mounted() {
    const videoElement = document.getElementById("video");

    this.initialize(videoElement);
    this.stateMsg = "Connecting...";
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
          ).then((remoteDescription) => {
            this.pc.setRemoteDescription(remoteDescription);
          });
        }
      };

      this.pc.oniceconnectionstatechange = async () => {
        if (this.pc.iceConnectionState == "disconnected") {
          console.log("Disconnected");

          window.setTimeout(() => {
            videoElement.pause();
            videoElement.removeAttribute("src");
            videoElement.load();

            this.videoResolution = null;
            this.stateMsg = "Reconnecting...";

            this.initialize(videoElement);
          }, 600);
        }
      };

      this.video_channel = this.pc.createDataChannel("video");
      this.control_channel = this.pc.createDataChannel("control");

      videoElement.addEventListener("loadedmetadata", (event) => {
        const width = event.target.videoWidth;
        const height = event.target.videoHeight;

        console.log("Video resolution is now:", width, "x", height, ".");
        this.videoResolution = { width: width, height: height };
        this.stateMsg = null;

        observer.observe(videoElement.parentElement);
      });

      const observer = new ResizeObserver(
        debounce(() => {
          // Ensure our resolution is a multiple of two.
          const width = videoElement.parentElement.offsetWidth & ~1;
          const height = videoElement.parentElement.offsetHeight & ~1;

          if (width == 0 || height == 0) return;

          console.log("Desired resolution is now:", width, "x", height, ".");
          this.desiredResolution = { width: width, height: height };

          this.stateMsg = "Resizing...";

          this.video_channel.send(
            JSON.stringify({
              event: "resize",
              width: width,
              height: height,
            })
          );
        }, 250)
      );

      this.video_channel.onerror = async (error) => {
        console.log(error.error);
      };
      this.video_channel.onopen = async () => {
        console.log("Video channel is now open.");
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
    backgroundURL() {
      var width = 1024;
      var height = 768;

      if (this.desiredResolution) {
        var { width, height } = this.desiredResolution;
      }

      return {
        "--bg-url":
          "url(https://source.unsplash.com/random/" +
          width +
          "x" +
          height +
          "?3d-render)",
      };
    },
  },
  watch: {
    resource(resource) {
      if (resource.description.id != "triangle") return;
      if (this.video_channel == null) return;
      if (this.video_channel.readyState != "open") return;

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
