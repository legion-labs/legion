import { onVideoClose } from "../api";
import log from "../lib/log";

type Source = MediaSource | SourceBuffer | HTMLVideoElement;

type Listener = {
  source: Source;
  name: string;
  f: () => void;
};

const sourceBufferType = 'video/mp4; codecs="avc1.640C34";';

/**
 * An augmented HTMLVideoElement videoElement.
 * Data received through the `push` method are displayed in the player.
 */
export type PushableHTMLVideoElement = HTMLVideoElement & {
  push(data: ArrayBuffer): void;
};

/**
 * Takes an `HTMLVideoElement` and attach a `push` method to it.
 *
 * The attached `push` method accepts an `ArrayBuffer` that will be
 * used as a frame in the video.
 * @param videoElement An `HTMLVideoElement`
 * @param options
 */
export default function videoPlayer(
  videoElement: HTMLVideoElement,
  options?: { onFatal?: () => void }
) {
  videoElement.muted = true;
  videoElement.disablePictureInPicture = true;

  const queue: Uint8Array[] = [];

  let videoSource: SourceBuffer | null = null;
  let mediaSource: MediaSource | null = null;
  let listeners: Listener[] = [];
  let lastFrameId = -1;

  function addListener(source: Source, name: string, f: () => void) {
    source.addEventListener(name, f);

    listeners.push({ source, name, f });
  }

  function initialize() {
    mediaSource = new MediaSource();

    videoElement.src = URL.createObjectURL(mediaSource);

    videoElement.load();

    addListener(videoElement, "error", () => {
      log.error("video", videoElement.error?.message);
    });

    addListener(mediaSource, "sourceopen", () => {
      videoSource = mediaSource?.addSourceBuffer(sourceBufferType) ?? null;

      if (videoSource) {
        addListener(videoSource, "update", submit);
      }

      videoElement.play().catch(() => {
        log.warn(
          "video",
          "Video player's pause method was called while the play method was running"
        );
      });
    });
  }

  function submit() {
    if (queue.length > 0 && videoSource && !videoSource.updating) {
      try {
        const frame = queue.shift();

        if (frame) {
          videoSource.appendBuffer(frame);
        }
      } catch (error) {
        log.error("video", error);

        options?.onFatal && options.onFatal();

        destroy();
      }
    }
  }

  function destroy() {
    onVideoClose();

    lastFrameId = -1;

    videoElement.pause();

    for (const { source, name, f } of listeners) {
      source.removeEventListener(name, f);
    }

    listeners = [];

    if (mediaSource) {
      if (videoSource) {
        mediaSource.removeSourceBuffer(videoSource);

        videoSource = null;
      }

      mediaSource.endOfStream();

      URL.revokeObjectURL(videoElement.src);

      mediaSource = null;
    }
  }

  (videoElement as PushableHTMLVideoElement).push = (data) => {
    const chunk = new Uint8Array(data);
    const frameId =
      (chunk[3] << 24) | (chunk[2] << 16) | (chunk[1] << 8) | chunk[0];
    const chunkCount =
      (chunk[7] << 24) | (chunk[6] << 16) | (chunk[5] << 8) | chunk[4];
    const chunkIdx =
      (chunk[11] << 24) | (chunk[10] << 16) | (chunk[9] << 8) | chunk[8];

    const frameChunk = chunk.slice(12);

    if (chunkIdx === 0 && frameChunk[4] === 0x66) {
      destroy();

      initialize();

      lastFrameId = frameId;
    }

    if (frameId < lastFrameId) {
      console.error("video", "Frame out of order");
      console.error(`Frame Id: ${frameId}`);
      console.error(`Chunk Count: ${chunkCount}`);
      console.error(`Chunk Idx: ${chunkIdx}`);
    }

    if (lastFrameId !== -1) {
      queue.push(frameChunk);

      submit();

      lastFrameId = frameId;
    }
  };
}
