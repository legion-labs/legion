<script context="module" lang="ts">
  export type Resolution = { width: number; height: number };
</script>

<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import resize from "@/actions/resize";
  import { debounce } from "@/lib/promises";
  import { addIceCandidates, initializeStream, iceCandidates } from "@/api";
  import statusBar from "@/stores/statusBar";

  const resizeVideoTimeout = 300;

  export let desiredResolution: Resolution | null = null;

  let resolution: Resolution | null = null;

  let videoElement: HTMLVideoElement;

  let videoChannel: RTCDataChannel | null;

  let controlChannel: RTCDataChannel | null;

  let peerConnection: RTCPeerConnection | null;

  let streamId: string | undefined;

  let videoAlreadyRendered = false;

  $statusBar = "Connecting...";

  onMount(() => {
    initialize();
  });

  onDestroy(() => {
    destroy();
  });

  async function onIceCandidate(event: RTCPeerConnectionIceEvent) {
    console.log("icecandidate", event.candidate);
  }

  function onIceConnectionStateChange(event: any) {
    console.log("iceconnectinstatechange", peerConnection?.iceConnectionState);
  }

  function onNegociationNeeded(event: any) {
    console.log("negotiationneeded");
  }

  async function onTrack(event: RTCTrackEvent) {
    console.log("ontrack", event);

    if (!videoElement) {
      return;
    }

    let mediaStream: MediaStream;

    if (event.streams[0]) {
      mediaStream = event.streams[0];
    } else {
      mediaStream = new MediaStream([event.track]);
    }

    console.log("set video src", mediaStream);

    videoElement.srcObject = mediaStream;

    videoElement.play();

    setTimeout(() => {
      videoElement.play();
    }, 1000);
  }

  // async function onVideoChannelMessage(
  //   event: MessageEvent<ArrayBuffer | Blob>
  // ) {
  //   let data: ArrayBuffer;

  //   if (event.data instanceof ArrayBuffer) {
  //     data = event.data;
  //   } else if (event.data instanceof Blob) {
  //     data = await event.data.arrayBuffer();
  //   }
  // }

  function destroy() {
    if (!peerConnection || !videoChannel) {
      // Peer Connection and related channels aren't initialized
      // so it's likely no event listeners have been added, skip the destroy.
      return;
    }

    peerConnection.removeEventListener("icecandidate", onIceCandidate);
    peerConnection.removeEventListener(
      "iceconnectionstatechange",
      onIceConnectionStateChange
    );
    peerConnection.removeEventListener(
      "negotiationneeded",
      onNegociationNeeded
    );
    peerConnection.removeEventListener("track", onTrack);
    // videoChannel.removeEventListener("message", onVideoChannelMessage);
  }

  async function initialize() {
    // Peer Connection
    peerConnection = new RTCPeerConnection({
      iceServers: [{ urls: ["stun:stun.l.google.com:19302"] }],
    });

    // Peer Connection event listeners
    peerConnection.addEventListener("icecandidate", onIceCandidate);

    peerConnection.addEventListener(
      "iceconnectionstatechange",
      onIceConnectionStateChange
    );

    peerConnection.addEventListener("negotiationneeded", onNegociationNeeded);

    peerConnection.addEventListener("track", onTrack);

    // Video Channel
    videoChannel = peerConnection.createDataChannel("video");

    // Video Channel event listeners
    // videoChannel.addEventListener("message", onVideoChannelMessage);

    // Control Channel
    controlChannel = peerConnection.createDataChannel("control");

    // Control Channel event listeners

    // Set transceiver
    peerConnection.addTransceiver("video");

    // Init code
    const localDescription = await peerConnection.createOffer();

    await peerConnection.setLocalDescription(localDescription);

    const response = await initializeStream(peerConnection.localDescription!);

    if (response.type === "ok") {
      await peerConnection.setRemoteDescription(response.sessionDescription);

      streamId = response.streamId;

      $statusBar = null;
    } else {
      console.log("An error occured:", response.error);
    }
  }

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

    // if (videoChannel && videoChannel.readyState === "open") {
    //   videoChannel.send(JSON.stringify({ event: "resize", width, height }));
    // }
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

    $statusBar = "Resizing...";
  }
</script>

<div class="video-container" use:resize={onVideoResize}>
  <video class="video" bind:this={videoElement} autoplay playsInline muted />
  {#if $statusBar}
    <h3 class="status">
      <span>{$statusBar}</span>
    </h3>
  {/if}
</div>

<style lang="postcss">
  .video-container {
    @apply h-full w-full overflow-hidden text-white;
  }

  .video {
    @apply inset-0 w-full h-full m-auto transition duration-200;
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
