<template>
  <div class="video-content">
    <video id="video"></video>
  </div>
</template>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped>
.video-content {
  height: 100vh;
}

video {
  cursor: pointer;
  background: url("~assets/images/disconnected.png") center center no-repeat;
  background-color: black;
  background-size: 20%;
  max-height: 100%;
  height: auto;
  width: 100%;
}
</style>

<script scoped>
import { invoke } from "@tauri-apps/api/tauri";

function addListener(obj, name, func, ctx) {
  const newFunc = ctx ? func.bind(ctx) : func;
  obj.addEventListener(name, newFunc);

  return [obj, name, newFunc];
}

function removeListeners(listeners) {
  for (const listener of listeners)
    listener[0].removeEventListener(listener[1], listener[2]);
}

class VideoPlayer {
  constructor(element, onFatal) {
    this.element = element;
    this.onFatal = onFatal;
    this.videoSource = null;
    this.mediaSource = null;
    this.waitingForKeyFrame = true;
    this.listeners = [];
    this.queue = [];
  }

  _submit() {
    if (
      this.queue.length > 0 &&
      this.videoSource &&
      !this.videoSource.updating
    ) {
      try {
        const frame = this.queue.shift();
        this.videoSource.appendBuffer(frame);
      } catch (error) {
        console.warn(error);
        this.destroy();
        this.onFatal();
      }
    }
  }

  _init() {
    this.mediaSource = new MediaSource();
    this.element.src = URL.createObjectURL(this.mediaSource);
    this.element.load();

    this.listeners.push(
      addListener(this.element, "error", () => {
        console.error(this.element.error.message);
      })
    );

    this.listeners.push(
      addListener(this.mediaSource, "sourceopen", () => {
        this.videoSource = this.mediaSource.addSourceBuffer(
          'video/mp4; codecs="avc1.640C34";'
        );
        this.listeners.push(
          addListener(this.videoSource, "update", this._submit, this)
        );
        this.element.play();
      })
    );
  }

  _reinit() {
    this.destroy();
    this._init();
  }

  destroy() {
    this.waitingForKeyFrame = true;
    this.element.pause();

    removeListeners(this.listeners);
    this.listeners = [];

    if (this.mediaSource) {
      if (this.videoSource) {
        this.mediaSource.removeSourceBuffer(this.videoSource);
        this.videoSource = null;
      }

      this.mediaSource.endOfStream();
      URL.revokeObjectURL(this.element.src);
      this.mediaSource = null;
    }
  }

  push(data) {
    const frame = new Uint8Array(data);

    if (frame[4] === 0x66) {
      this._reinit();
      this.waitingForKeyFrame = false;
    }

    if (!this.waitingForKeyFrame) {
      this.queue.push(frame);
      this._submit();
    }
  }
}

export default {
  name: "Video",
  props: {
    msg: String,
  },
  mounted() {
    const videoElement = document.getElementById("video");
    const videoPlayer = new VideoPlayer(videoElement, () => {});

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

      video_channel.onmessage = async (msg) => {
        videoPlayer.push(msg.data);
      };

      video_channel.onmessage = async (msg) => {
        videoPlayer.push(msg.data);
      };
    };
  },
};
</script>