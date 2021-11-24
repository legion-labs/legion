import { onVideoChunkReceived, onVideoClose } from "@/api";

type Source = MediaSource | SourceBuffer | HTMLVideoElement;

type Listener = {
  source: Source;
  name: string;
  f: () => void;
};

/**
 * An augmented HTMLVideoElement node.
 * Data received through the `push` method are displayed in the player.
 */
export type PushableHTMLVideoElement = HTMLVideoElement & {
  push(data: ArrayBuffer): void;
};

export default function videoPlayer(
  node: HTMLVideoElement,
  options?: { onFatal?: () => void }
) {
  if (!(node instanceof HTMLVideoElement)) {
    throw new Error(
      "Target node for `videoPlayer` should be an instance of `HTMLVideoElement`"
    );
  }

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
    node.src = URL.createObjectURL(mediaSource);
    node.load();

    addListener(node, "error", () => {
      console.error(node.error?.message);
    });

    addListener(mediaSource, "sourceopen", () => {
      videoSource =
        mediaSource?.addSourceBuffer('video/mp4; codecs="avc1.640C34";') ??
        null;

      if (videoSource) {
        addListener(videoSource, "update", submit);
      }

      node.play();
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
        console.warn(error);
        destroy();
        options?.onFatal && options.onFatal();
      }
    }
  };

  const destroy = () => {
    onVideoClose();

    waitingForKeyFrame = true;
    node.pause();

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
      URL.revokeObjectURL(node.src);
      mediaSource = null;
    }
  };

  (node as PushableHTMLVideoElement).push = (data) => {
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
