import { onVideoChunkReceived, onVideoClose } from "../api";
import log from "../lib/log";

type Source = MediaSource | SourceBuffer | HTMLVideoElement;

type Listener = {
  source: Source;
  name: string;
  f: () => void;
};

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
  let videoSource: SourceBuffer | null = null;
  let mediaSource: MediaSource | null = null;
  let waitingForKeyFrame = true;
  let listeners: Listener[] = [];
  const queue: Uint8Array[] = [];

  const addListener = (source: Source, name: string, f: () => void) => {
    source.addEventListener(name, f);

    listeners.push({ source, name, f });
  };

  const initialize = () => {
    mediaSource = new MediaSource();
    videoElement.src = URL.createObjectURL(mediaSource);
    videoElement.load();

    addListener(videoElement, "error", () => {
      log.error("video", videoElement.error?.message);
    });

    addListener(mediaSource, "sourceopen", () => {
      videoSource =
        mediaSource?.addSourceBuffer('video/mp4; codecs="avc1.640C34";') ??
        null;

      if (videoSource) {
        addListener(videoSource, "update", submit);
      }

      videoElement.play();
    });
  };

  const submit = () => {
    if (queue.length > 0 && videoSource && !videoSource.updating) {
      try {
        const frame = queue.shift();

        if (frame) {
          videoSource.appendBuffer(frame);
        }
      } catch (error) {
        log.error("video", error);
        destroy();
        options?.onFatal && options.onFatal();
      }
    }
  };

  const destroy = () => {
    onVideoClose();

    waitingForKeyFrame = true;
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
  };

  (videoElement as PushableHTMLVideoElement).push = (data) => {
    const chunk = new Uint8Array(data);
    const headerPayloadLength = chunk[1] * 256 + chunk[0];
    const binHeader = chunk.slice(2, 2 + headerPayloadLength);

    const chunkHeader = new TextDecoder().decode(binHeader);

    onVideoChunkReceived(chunkHeader);

    const frame = chunk.slice(2 + headerPayloadLength);

    if (frame[4] === 0x66) {
      destroy();
      initialize();
      waitingForKeyFrame = false;
    }

    if (!waitingForKeyFrame) {
      queue.push(frame);
      submit();
    }
  };
}
