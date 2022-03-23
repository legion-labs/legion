<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { debounce, retry } from "../lib/promises";
  import statusStore from "../stores/statusBarData";
  import {
    initializeStream,
    onReceiveControlMessage,
    ServerType,
  } from "../api";
  import log from "../lib/log";
  import type { PushableHTMLVideoElement } from "../actions/videoPlayer";
  import resize from "../actions/resize";
  import videoPlayer from "../actions/videoPlayer";
  import remoteWindowInputs, {
    RemoteWindowInput,
  } from "../actions/remoteWindowInputs";
  import type { Resolution } from "../lib/types";

  const reconnectionTimeout = 1_000;

  const resizeVideoTimeout = 300;

  const connectionRetry = 1_000;

  const backgroundColors = {
    editor: "#000066",
    runtime: "#112211",
  };

  export let desiredResolution: Resolution | null = null;

  export let serverType: ServerType;

  let resolution: Resolution | null = null;

  let videoElement: HTMLVideoElement | undefined;

  let videoChannel: RTCDataChannel | null;

  let controlChannel: RTCDataChannel | null;

  let peerConnection: RTCPeerConnection | null;

  let videoAlreadyRendered = false;

  let reconnectionIntervalId: ReturnType<typeof setInterval> | null = null;

  $statusStore = "Connecting...";

  $: loading = !!$statusStore;

  onMount(() => {
    initialize();
  });

  onDestroy(() => {
    destroyResources();
  });

  // Destroys all peer connection related resources when possible
  function destroyResources() {
    if (videoChannel) {
      videoChannel.close();
      videoChannel = null;
    }

    if (controlChannel) {
      controlChannel.close();
      controlChannel = null;
    }

    if (peerConnection) {
      peerConnection.close();
      peerConnection = null;
    }

    if (reconnectionIntervalId) {
      clearInterval(reconnectionIntervalId);
    }
  }

  async function initialize() {
    if (!videoElement) {
      log.error("video", "Video element couldn't be found");

      return;
    }

    log.debug("video", "Initializing WebRTC");

    peerConnection = new RTCPeerConnection({
      iceServers: [{ urls: ["stun:stun.l.google.com:19302"] }],
    });

    videoChannel = peerConnection.createDataChannel("video");

    videoChannel.binaryType = "arraybuffer";

    controlChannel = peerConnection.createDataChannel("control");

    videoChannel.binaryType = "arraybuffer";

    peerConnection.onnegotiationneeded = async () => {
      if (!peerConnection) {
        return;
      }

      peerConnection.setLocalDescription(await peerConnection.createOffer());

      const remoteDescription = await retry(
        debounce(() => {
          if (peerConnection && peerConnection.localDescription) {
            return initializeStream(
              serverType,
              peerConnection.localDescription
            );
          }

          return Promise.resolve(null);
        }, 1_000),
        connectionRetry
      );

      if (remoteDescription) {
        peerConnection.setRemoteDescription(remoteDescription);
      } else {
        log.error("video", "Server didn't return any SDP description");
      }
    };

    peerConnection.onicecandidate = async (iceEvent) => {
      // TODO: Handle proper ice candidates exchange:
      // https://developer.mozilla.org/en-US/docs/Web/API/WebRTC_API/Signaling_and_video_calling/webrtc_-_ice_candidate_exchange.svg
      log.debug("video", iceEvent);
    };

    peerConnection.oniceconnectionstatechange = () => {
      if (
        peerConnection &&
        peerConnection.iceConnectionState === "disconnected"
      ) {
        log.debug("video", "Disconnected");

        reconnectionIntervalId = setInterval(() => {
          $statusStore = "Reconnecting...";

          loading = true;

          destroyResources();

          initialize();
        }, reconnectionTimeout);
      }
    };

    videoElement.addEventListener("loadedmetadata", (event) => {
      if (videoElement && event.target instanceof HTMLVideoElement) {
        if (!videoAlreadyRendered) {
          videoAlreadyRendered = true;
        }

        const { videoWidth, videoHeight } = event.target;

        log.debug(
          "video",
          `Video resolution is now: ${videoWidth}x${videoHeight}.`
        );

        $statusStore = null;

        resolution = desiredResolution;
      }
    });

    videoChannel.onerror = (error: unknown) => {
      log.error("video", error);
    };

    videoChannel.onopen = () => {
      if (!videoChannel || !desiredResolution) {
        log.error("video", "Video channel couldn't be open.");

        return;
      }

      const normalizedDesiredResolution =
        normalizeDesiredResolution(desiredResolution);

      if (
        !normalizedDesiredResolution ||
        normalizedDesiredResolution.width == 0 ||
        normalizedDesiredResolution.height == 0
      ) {
        return;
      }

      log.debug(
        "video",
        `Resolution is: ${normalizedDesiredResolution.width}x${normalizedDesiredResolution.height}`
      );

      videoChannel.send(
        JSON.stringify({
          event: "initialize",
          color: backgroundColors[serverType],
          width: normalizedDesiredResolution.width,
          height: normalizedDesiredResolution.height,
        })
      );

      log.debug("video", "Video channel is now open.");
    };

    videoChannel.onclose = () => {
      log.debug("video", "Video channel is now closed.");
    };

    videoChannel.onmessage = async (message: MessageEvent<ArrayBuffer>) => {
      if (!videoElement) {
        return;
      }

      // videoElement is augmented with the `videoPlayer` action and will
      // provide a `push` function.
      (videoElement as PushableHTMLVideoElement).push(message.data);
    };

    controlChannel.onopen = (event) => {
      log.debug("video", log.json`Control channel is now open: ${event}`);
    };

    controlChannel.onclose = (event) => {
      log.debug("video", log.json`Control channel is now closed: ${event}`);
    };

    controlChannel.onmessage = async (message: MessageEvent<unknown>) => {
      const jsonMsg =
        message.data instanceof ArrayBuffer
          ? new TextDecoder().decode(message.data)
          : // TODO: Refine data type
            // eslint-disable-next-line @typescript-eslint/no-explicit-any
            (message.data as any);

      onReceiveControlMessage(jsonMsg);
    };
  }

  const resizeVideo = debounce((desiredResolution: Resolution) => {
    if (!videoAlreadyRendered) {
      return;
    }

    const normalizedDesiredResolution =
      normalizeDesiredResolution(desiredResolution);

    if (
      !normalizedDesiredResolution ||
      normalizedDesiredResolution.width == 0 ||
      normalizedDesiredResolution.height == 0
    ) {
      return;
    }

    log.debug(
      "video",
      `Desired resolution is now: ${normalizedDesiredResolution.width}x${normalizedDesiredResolution.height}`
    );

    if (videoChannel && videoChannel.readyState === "open") {
      videoChannel.send(
        JSON.stringify({
          event: "resize",
          width: normalizedDesiredResolution.width,
          height: normalizedDesiredResolution.height,
        })
      );
    }
  }, resizeVideoTimeout);

  function onVideoResize({ width, height }: DOMRectReadOnly) {
    desiredResolution = {
      width: Math.round(width),
      height: Math.round(height),
    };
  }

  function onRemoteWindowInput(input: RemoteWindowInput) {
    if (!videoChannel || videoChannel.readyState !== "open") {
      log.debug(
        "video remote window",
        "Received an input while the video channel wasn't available"
      );

      return;
    }

    videoChannel.send(JSON.stringify({ event: "input", input }));
  }

  function normalizeDesiredResolution(
    desiredResolution: Resolution
  ): Resolution | null {
    // Ensure our resolution is a multiple of two.
    const width = desiredResolution.width & ~1;
    const height = desiredResolution.height & ~1;

    if (width == 0 || height == 0) {
      return null;
    }

    return { width, height };
  }

  $: if (
    resolution &&
    desiredResolution &&
    (resolution.height !== desiredResolution.height ||
      resolution.width !== desiredResolution.width)
  ) {
    $statusStore = "Resizing...";

    resizeVideo(desiredResolution);
  }
</script>

<div
  class="video-container"
  use:resize={onVideoResize}
  use:remoteWindowInputs={onRemoteWindowInput}
>
  <video class="video" use:videoPlayer bind:this={videoElement}>
    <track kind="captions" />
  </video>
  <!-- TODO: Set opacity to 70 or so to still see the video player, blinks for the moment -->
  <div
    class="loading-overlay"
    class:opacity-0={!loading}
    class:opacity-100={loading}
  >
    {#if $statusStore}
      <div class="status">
        {$statusStore}
      </div>
    {/if}
  </div>
</div>

<style lang="postcss">
  .video-container {
    @apply h-full w-full overflow-hidden relative text-white;
  }

  .video {
    @apply absolute object-cover inset-0 w-full h-full m-auto transition duration-200;
  }

  .loading-overlay {
    @apply absolute w-full h-full bg-gray-800;
  }

  .status {
    @apply flex flex-row items-center justify-center h-full animate-pulse text-xl;
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
