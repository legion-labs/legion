<script>
import {
  initializeStream,
  onReceiveControlMessage,
  onSendEditionCommand,
} from "~/modules/api";

import { VideoPlayer } from "~/modules/video";
import { debounce, retryForever } from "~/modules/futures";

export default {
  name: "Video",
  props: {
    // eslint-disable-next-line vue/require-default-prop
    resource: null,
  },
  data() {
    return {
      videoResolution: null,
      desiredResolution: null,
      pc: null,
      videoChannel: null,
      controlChannel: null,
      stateMsg: null,
    };
  },
  computed: {
    loading() {
      return (
        !this.videoResolution ||
        !this.desiredResolution ||
        this.videoResolution.width !== this.desiredResolution.width ||
        this.videoResolution.height !== this.desiredResolution.height
      );
    },
  },
  watch: {
    resource(resource) {
      if (resource.description.id !== "triangle") return;
      if (this.videoChannel == null) return;
      if (this.videoChannel.readyState !== "open") return;

      for (const property of resource.properties) {
        let editionEvent = null;

        if (property.name === "color") {
          editionEvent = JSON.stringify({
            id: crypto.randomUUID(),
            event: "color",
            color: property.value,
          });
        } else if (property.name === "speed") {
          editionEvent = JSON.stringify({
            id: crypto.randomUUID(),
            event: "speed",
            speed: property.value,
          });
        }

        if (editionEvent) {
          onSendEditionCommand(editionEvent);
          this.videoChannel.send(editionEvent);
        }
      }
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

      if (this.videoChannel != null) {
        this.videoChannel.close();
        this.videoChannel = null;
      }

      if (this.controlChannel != null) {
        this.controlChannel.close();
        this.controlChannel = null;
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

      this.pc.onicecandidate = (iceEvent) => {
        console.log(iceEvent);

        if (iceEvent.candidate === null) {
          retryForever(
            initializeStream.bind(null, this.pc.localDescription)
          ).then((remoteDescription) => {
            this.pc.setRemoteDescription(remoteDescription);
          });
        }
      };

      this.pc.oniceconnectionstatechange = () => {
        if (this.pc.iceConnectionState === "disconnected") {
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

      this.videoChannel = this.pc.createDataChannel("video");
      this.controlChannel = this.pc.createDataChannel("control");

      videoElement.addEventListener("loadedmetadata", (event) => {
        const width = event.target.videoWidth;
        const height = event.target.videoHeight;

        console.log("Video resolution is now:", width, "x", height, ".");
        this.videoResolution = { width, height };
        this.stateMsg = null;

        observer.observe(videoElement.parentElement);
      });

      const observer = new ResizeObserver(
        debounce(() => {
          // Ensure our resolution is a multiple of two.
          const width = videoElement.parentElement.offsetWidth & ~1;
          const height = videoElement.parentElement.offsetHeight & ~1;

          if (width === 0 || height === 0) return;

          console.log("Desired resolution is now:", width, "x", height, ".");
          this.desiredResolution = { width, height };

          this.stateMsg = "Resizing...";

          this.videoChannel.send(
            JSON.stringify({
              event: "resize",
              width,
              height,
            })
          );
        }, 250)
      );

      this.videoChannel.onerror = (error) => {
        console.error(error.error);
      };
      this.videoChannel.onopen = () => {
        console.log("Video channel is now open.");
      };
      this.videoChannel.onclose = () => {
        console.log("Video channel is now closed.");
        observer.disconnect();
      };
      this.videoChannel.onmessage = (msg) => {
        videoPlayer.push(msg.data);
      };
      this.videoChannel.ondatachannel = (evt) => {
        console.log("video data channel: ", evt);
      };
      this.controlChannel.onopen = (evt) => {
        console.log("Control channel is now open: ", evt);
      };
      this.controlChannel.onclose = (evt) => {
        console.log("Control channel is now closed: ", evt);
      };
      this.controlChannel.ondatachannel = (evt) => {
        console.log("control data channel: ", evt);
      };
      this.controlChannel.onmessage = (msg) => {
        const jsonMsg = new TextDecoder().decode(msg.data);
        onReceiveControlMessage(jsonMsg);
      };
    },
    backgroundURL() {
      let width = 1024;
      let height = 768;

      if (this.desiredResolution) {
        width = this.desiredResolution.width;
        height = this.desiredResolution.height;
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
};
</script>

<template>
  <div id="video-container" :style="backgroundURL()" class="d-flex">
    <video id="video" :class="{ show: !loading }"></video>
    <v-progress-linear
      v-if="loading"
      id="loading"
      indeterminate
      color="yellow darken-2"
    ></v-progress-linear>
    <code v-if="videoResolution" id="resolution" :class="{ show: !loading }">
      {{ videoResolution.width }}x{{ videoResolution.height }}</code
    >
    <h3 v-if="stateMsg" id="connecting">
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
