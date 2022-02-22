<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { debounce, retry } from "../lib/promises";
  import { statusStore } from "../stores/statusBarData";
  import {
    initializeStream,
    onReceiveControlMessage,
    ServerType,
  } from "../api";
  import log from "../lib/log";
  import { PushableHTMLVideoElement } from "../actions/videoPlayer";
  import resize from "../actions/resize";
  import videoPlayer from "../actions/videoPlayer";
  import remoteWindowInputs, {
    RemoteWindowInput,
  } from "../actions/remoteWindowInputs";
  import { Resolution } from "../lib/types";

  const reconnectionTimeout = 600;

  const resizeVideoTimeout = 300;

  const connectionRetry = 1000;

  export let desiredResolution: Resolution | null = null;

  export let serverType: ServerType;

  let resolution: Resolution | null = null;

  let videoElement: HTMLVideoElement | undefined;

  let videoChannel: RTCDataChannel | null;

  let controlChannel: RTCDataChannel | null;

  let peerConnection: RTCPeerConnection | null;

  let videoAlreadyRendered = false;

  let loading = false;

  $statusStore = "Connecting...";

  const backgroundColors = {
    editor: "#000066",
    runtime: "#112211",
  };

  onMount(() => {
    initialize();
  });

  onDestroy(() => {
    destroyResources();
  });

  // Destroys all peer connection related resources when possible
  const destroyResources = () => {
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
  };

  function initialize() {
    if (!videoElement) {
      log.error("video", "Video element couldn't be found");

      return;
    }

    log.debug("video", "Initializing WebRTC");

    peerConnection = new RTCPeerConnection({
      iceServers: [{ urls: ["stun:stun.l.google.com:19302"] }],
    });

    peerConnection.onnegotiationneeded = async () => {
      if (peerConnection) {
        peerConnection.setLocalDescription(await peerConnection.createOffer());
      }
    };

    peerConnection.onicecandidate = async (iceEvent) => {
      log.debug("video", iceEvent);

      if (peerConnection && iceEvent.candidate === null) {
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
        }
      }
    };

    peerConnection.oniceconnectionstatechange = () => {
      if (
        peerConnection &&
        peerConnection.iceConnectionState === "disconnected"
      ) {
        log.debug("video", "Disconnected");

        window.setTimeout(() => {
          if (videoElement) {
            videoElement.pause();
          }
          $statusStore = "Reconnecting...";

          destroyResources();

          initialize();
        }, reconnectionTimeout);
      }
    };

    videoChannel = peerConnection.createDataChannel("video");

    videoChannel.binaryType = "arraybuffer";

    controlChannel = peerConnection.createDataChannel("control");

    videoChannel.binaryType = "arraybuffer";

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

        loading = false;
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
    resizeVideo(desiredResolution);
    $statusStore = "Resizing...";
    loading = true;
  }
</script>

<div
  class="video-container"
  use:resize={onVideoResize}
  use:remoteWindowInputs={onRemoteWindowInput}
>
  <video
    class="video"
    class:opacity-0={loading}
    class:opacity-100={!loading}
    use:videoPlayer
    bind:this={videoElement}
  >
    <track kind="captions" />
  </video>
  {#if $statusStore}
    <h3 class="status">
      <span>{$statusStore}</span>
    </h3>
  {/if}
</div>

<style lang="postcss">
  .video-container {
    @apply h-full w-full overflow-hidden relative text-white;
  }

  .video {
    @apply absolute object-cover inset-0 w-full h-full m-auto transition duration-200;
  }

  .status {
    @apply absolute left-0 right-0 top-1/2 text-center;
    animation: glow 1s infinite alternate;
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
