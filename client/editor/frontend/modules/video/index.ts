import { onVideoChunkReceived, onVideoClose } from "../api";

type Source = HTMLVideoElement | MediaSource | SourceBuffer;

type Listener = {
  source: Source;
  name: string;
  // eslint-disable-next-line no-use-before-define
  f: (this: VideoPlayer) => void;
};

export class VideoPlayer {
  private videoSource: SourceBuffer | null;
  private mediaSource: MediaSource | null;
  private waitingForKeyFrame: boolean;
  private listeners: Listener[];
  private queue: BufferSource[];

  constructor(
    private element: HTMLVideoElement,
    private onFatal = () => {
      // Ignore fatal errors
    }
  ) {
    this.videoSource = null;
    this.mediaSource = null;
    this.waitingForKeyFrame = true;
    this.listeners = [];
    this.queue = [];
  }

  public push(data: Iterable<number>) {
    const chunk = new Uint8Array(data);
    const headerPayloadLen = chunk[1] * 256 + chunk[0];
    const binHeader = chunk.slice(2, 2 + headerPayloadLen);
    const chunkHeader = new TextDecoder().decode(binHeader);

    onVideoChunkReceived(chunkHeader);

    const frame = chunk.slice(2 + headerPayloadLen);

    if (frame[4] === 0x66) {
      this.reinit();
      this.waitingForKeyFrame = false;
    }

    if (!this.waitingForKeyFrame) {
      this.queue.push(frame);
      this.submit();
    }
  }

  private submit() {
    if (
      this.queue.length > 0 &&
      this.videoSource &&
      !this.videoSource.updating
    ) {
      try {
        const frame = this.queue.shift();

        if (frame) {
          this.videoSource.appendBuffer(frame);
        }
      } catch (error) {
        console.warn(error);
        this.destroy();
        this.onFatal();
      }
    }
  }

  private init() {
    this.mediaSource = new MediaSource();
    this.element.src = URL.createObjectURL(this.mediaSource);
    this.element.load();

    this.pushListener(this.element, "error", () => {
      console.error(this.element.error?.message);
    });

    this.pushListener(this.mediaSource, "sourceopen", () => {
      this.videoSource =
        this.mediaSource?.addSourceBuffer('video/mp4; codecs="avc1.640C34";') ??
        null;

      if (this.videoSource) {
        this.pushListener(this.videoSource, "update", this.submit.bind(this));
      }

      this.element.play();
    });
  }

  private reinit() {
    this.destroy();
    this.init();
  }

  private pushListener(
    source: Source,
    name: string,
    f: (this: VideoPlayer) => void
  ): void {
    source.addEventListener(name, f);

    this.listeners.push({ source, name, f });
  }

  private removeListeners() {
    for (const { source, name, f } of this.listeners) {
      source.removeEventListener(name, f);
    }

    this.listeners = [];
  }

  private destroy() {
    onVideoClose();

    this.waitingForKeyFrame = true;
    this.element.pause();

    this.removeListeners();

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
}
