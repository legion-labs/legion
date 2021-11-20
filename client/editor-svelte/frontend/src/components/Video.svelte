<script lang="ts">
  import { onMount } from "svelte";
  import {
    initializeStream,
    onReceiveControlMessage,
    onSendEditionCommand,
    VideoPlayer,
    debounce,
    retryForever,
  } from "@/lib";

  export let resource: any = null;

  let videoElement: any = null;

  let videoResolution: any = null;

  let desiredResolution: any = null;

  let videoChannel: any = null;

  let controlChannel: any = null;

  let stateMsg: any = null;

  let pc: any = null;

  onMount(() => {
    initialize(videoElement);

    stateMsg = "Connecting...";
  });

  const initialize = (videoElement: HTMLVideoElement) => {
    const videoPlayer = new VideoPlayer(videoElement);

    console.log("Initializing WebRTC...");

    if (videoChannel !== null) {
      videoChannel.close();
      videoChannel = null;
    }

    if (controlChannel !== null) {
      controlChannel.close();
      controlChannel = null;
    }

    if (pc !== null) {
      pc.close();
      pc = null as any;
    }

    pc = new RTCPeerConnection({
      urls: [{ url: "stun:stun.l.google.com:19302" }],
    } as any);

    pc.onnegotiationneeded = async () => {
      pc.setLocalDescription(await pc.createOffer());
    };

    pc.onicecandidate = async (iceEvent: any) => {
      console.log(iceEvent);

      if (iceEvent.candidate === null) {
        retryForever(() => initializeStream(pc.localDescription!)).then(
          (remoteDescription) => {
            pc.setRemoteDescription(remoteDescription);
          }
        );
      }
    };

    pc.oniceconnectionstatechange = async () => {
      if (pc.iceConnectionState === "disconnected") {
        console.log("Disconnected");

        window.setTimeout(() => {
          videoElement.pause();
          videoElement.removeAttribute("src");
          videoElement.load();

          videoResolution = null;
          stateMsg = "Reconnecting...";

          initialize(videoElement);
        }, 600);
      }
    };

    videoChannel = pc.createDataChannel("video");
    controlChannel = pc.createDataChannel("control");

    videoElement.addEventListener("loadedmetadata", (event) => {
      const width = (event.target! as any).videoWidth;
      const height = (event.target! as any).videoHeight;

      console.log("Video resolution is now:", width, "x", height, ".");
      videoResolution = { width: width, height: height };
      stateMsg = null;

      observer.observe(videoElement.parentElement!);
    });

    const observer = new ResizeObserver(
      debounce(() => {
        // Ensure our resolution is a multiple of two.
        const width = videoElement.parentElement!.offsetWidth & ~1;
        const height = videoElement.parentElement!.offsetHeight & ~1;

        if (width == 0 || height == 0) {
          return;
        }

        console.log("Desired resolution is now:", width, "x", height, ".");
        desiredResolution = { width: width, height: height };

        stateMsg = "Resizing...";

        videoChannel.send(
          JSON.stringify({
            event: "resize",
            width: width,
            height: height,
          })
        );
      }, 250)
    );

    videoChannel.onerror = (error: any) => {
      console.log(error.error);
    };

    videoChannel.onopen = () => {
      console.log("Video channel is now open.");
    };

    videoChannel.onclose = () => {
      console.log("Video channel is now closed.");
      observer.disconnect();
    };

    videoChannel.onmessage = (msg: any) => {
      videoPlayer.push(msg.data);
    };

    videoChannel.ondatachannel = (event: any) => {
      console.log("video data channel: ", event);
    };

    controlChannel.onopen = (event: any) => {
      console.log("Control channel is now open: ", event);
    };

    controlChannel.onclose = (event: any) => {
      console.log("Control channel is now closed: ", event);
    };

    controlChannel.ondatachannel = (event: any) => {
      console.log("control data channel: ", event);
    };

    controlChannel.onmessage = (msg: any) => {
      // const jsonMsg = new TextDecoder().decode(msg.data);

      onReceiveControlMessage(msg);
    };
  };

  const backgroundURL = () => {
    let width = 1024;
    let height = 768;

    if (desiredResolution) {
      width = desiredResolution.width;
      height = desiredResolution.height;
    }

    return `--bg-url: url(https://source.unsplash.com/random/${width}x${height}?3d-render)`;
  };

  $: (() => {
    if (
      !resource ||
      resource.description.id !== "triangle" ||
      videoChannel === null ||
      videoChannel.readyState !== "open"
    ) {
      return;
    }

    for (const property of resource.properties) {
      let editionEvent = null;

      switch (property.name) {
        case "color": {
          editionEvent = JSON.stringify({
            id: (crypto as any).randomUUID(),
            event: "color",
            color: property.value,
          });

          break;
        }

        case "speed": {
          editionEvent = JSON.stringify({
            id: (crypto as any).randomUUID(),
            event: "speed",
            speed: property.value,
          });

          break;
        }
      }

      if (editionEvent) {
        onSendEditionCommand(editionEvent);

        videoChannel.send(editionEvent);
      }
    }
  })();

  $: loading =
    !videoResolution ||
    !desiredResolution ||
    videoResolution.width !== desiredResolution.width ||
    videoResolution.height !== desiredResolution.height;
</script>

<div
  id="video-container"
  class="h-full w-full bg-black overflow-hidden relative"
  style={backgroundURL()}
>
  <!-- svelte-ignore a11y-media-has-caption -->
  <video
    id="video"
    class="absolute object-cover inset-0 max-w-full max-h-full opacity-0 m-auto"
    class:show={!loading}
    bind:this={videoElement}
  />
  <!-- <v-progress-linear
    id="loading"
    indeterminate
    color="yellow darken-2"
    v-if="loading"
  /> -->
  {#if videoResolution}
    <code
      id="resolution"
      class="absolute rounded-2xl bg-gray-400 text-black top-4 right-4 opacity-0 text-sm"
      class:show={!loading}
    >
      {videoResolution.width}x{videoResolution.height}</code
    >
  {/if}
  {#if stateMsg}
    <h3
      id="connecting"
      class="absolute left-0 right-0 top-1/2 text-center text-gray-400"
    >
      <!-- <v-progress-circular indeterminate color="white" :size="30" /> -->
      <span>{stateMsg}</span>
    </h3>
  {/if}
</div>

<style>
  #video-container {
    background: linear-gradient(
        to top right,
        rgba(100, 115, 201, 0.7),
        rgba(25, 32, 72, 0.7)
      ),
      no-repeat center/cover var(--bg-url);
  }

  #video {
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
    /* border-radius: 1em; */
    transition: opacity 0.5s linear;
  }

  #resolution.show {
    opacity: 0.5;
  }

  #connecting {
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
