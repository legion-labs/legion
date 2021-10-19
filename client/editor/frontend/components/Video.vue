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
  on_video_close,
  on_video_chunk_received,
} from "~/modules/api";

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
    on_video_close();

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
    const chunk = new Uint8Array(data);
    const header_payload_len = chunk[1] * 256 + chunk[0];
    const bin_header = chunk.slice(2, 2 + header_payload_len);

    const chunkHeader = new TextDecoder().decode(bin_header);
    on_video_chunk_received(chunkHeader);

    const frame = chunk.slice(2 + header_payload_len);

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

function debounce(func, wait, immediate) {
  var timeout;

  return function () {
    var context = this,
      args = arguments;
    var later = function () {
      timeout = null;
      if (!immediate) func.apply(context, args);
    };
    var callNow = immediate && !timeout;
    clearTimeout(timeout);
    timeout = setTimeout(later, wait);
    if (callNow) func.apply(context, args);
  };
}

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

    videoElement.parentElement.onclick = () => {
      this.initialize(videoElement);
    };

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
          this.pc.setRemoteDescription(
            await initialize_stream(this.pc.localDescription)
          );
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
