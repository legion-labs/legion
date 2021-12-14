<script context="module" lang="ts">
  export type Resolution = { width: number; height: number };
</script>

<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import resize from "@/actions/resize";
  import videoPlayer, { PushableHTMLVideoElement } from "@/actions/videoPlayer";
  import { debounce, retry } from "@/lib/promises";
  import { initializeStream, onReceiveControlMessage, ServerType } from "@/api";
  import { statusStore } from "@/stores/statusBarData";
  import log from "@/lib/log";

  const reconnectionTimeout = 600;

  const resizeVideoTimeout = 300;

  const connectionRetry = 5;

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

  onMount(() => {
    initialize();
  });

  onDestroy(() => {
    destroyResources();
  });

  // Destroys all peer connection related resources when possible
  const destroyResources = () => {
    if (videoChannel !== null) {
      videoChannel.close();
      videoChannel = null;
    }

    if (controlChannel !== null) {
      controlChannel.close();
      controlChannel = null;
    }

    if (peerConnection !== null) {
      peerConnection.close();
      peerConnection = null;
    }
  };

  const initialize = () => {
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
        const remoteDescription = await retry(() => {
          if (peerConnection && peerConnection.localDescription) {
            return initializeStream(
              serverType,
              peerConnection.localDescription
            );
          }

          return Promise.resolve(null);
        }, connectionRetry);

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
            videoElement.removeAttribute("src");
            videoElement.load();
          }

          $statusStore = "Reconnecting...";

          destroyResources();

          initialize();
        }, reconnectionTimeout);
      }
    };

    videoChannel = peerConnection.createDataChannel("video");

    controlChannel = peerConnection.createDataChannel("control");

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
      log.debug("video", "Video channel is now open.");
    };

    videoChannel.onclose = () => {
      log.debug("video", "Video channel is now closed.");
    };

    videoChannel.onmessage = async (message) => {
      if (!videoElement) {
        return;
      }

      // videoElement is augmented with the `videoPlayer` action and will
      // provide a `push` function.
      (videoElement as PushableHTMLVideoElement).push(
        // In Tauri message.data is an ArrayBuffer
        // while it's a Blob in browser
        message.data instanceof ArrayBuffer
          ? message.data
          : await message.data.arrayBuffer()
      );
    };

    controlChannel.onopen = (event) => {
      log.debug("video", log.json`Control channel is now open: ${event}`);
    };

    controlChannel.onclose = (event) => {
      log.debug("video", log.json`Control channel is now closed: ${event}`);
    };

    controlChannel.onmessage = async (
      message: MessageEvent<ArrayBuffer | Blob>
    ) => {
      const jsonMsg = new TextDecoder().decode(
        // In Tauri message.data is an ArrayBuffer
        // while it's a Blob in browser
        message.data instanceof ArrayBuffer
          ? message.data
          : await message.data.arrayBuffer()
      );

      onReceiveControlMessage(jsonMsg);
    };
  };

  const resizeVideo = debounce((desiredResolution: Resolution) => {
    if (!videoAlreadyRendered) {
      return;
    }

    // Ensure our resolution is a multiple of two.
    const height = desiredResolution.height & ~1;
    const width = desiredResolution.width & ~1;

    if (width == 0 || height == 0) {
      return;
    }

    log.debug("video", `Desired resolution is now: ${width}x${height}`);

    if (videoChannel && videoChannel.readyState === "open") {
      videoChannel.send(JSON.stringify({ event: "resize", width, height }));
    }
  }, resizeVideoTimeout);

  const onVideoResize = ({ width, height }: DOMRectReadOnly) => {
    desiredResolution = {
      width: Math.round(width),
      height: Math.round(height),
    };
  };

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

<div class="video-container" use:resize={onVideoResize}>
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
