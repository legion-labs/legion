// TODO: Turn this into a store and add a batch command for proto
import { onVideoChunkReceived, onVideoClose } from "@/api";

type Source = MediaSource | SourceBuffer | HTMLVideoElement;

type Listener = {
  source: Source;
  name: string;
  f: (this: VideoPlayer) => void;
};

export class VideoPlayer {
  private videoSource: SourceBuffer | null = null;
  private mediaSource: MediaSource | null = null;
  private waitingForKeyFrame = true;
  private listeners: Listener[] = [];
  private queue: Uint8Array[] = [];

  constructor(
    private element: HTMLVideoElement,
    private onFatal = () => {
      // Ignore fatal errors
    }
  ) {}

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

  private addListener(source: Source, name: string, f: () => void) {
    source.addEventListener(name, f);

    this.listeners.push({ source, name, f });
  }

  private initialize() {
    this.mediaSource = new MediaSource();
    this.element.src = URL.createObjectURL(this.mediaSource);
    this.element.load();

    this.addListener(this.element, "error", () => {
      console.error(this.element.error?.message);
    });

    this.addListener(this.mediaSource, "sourceopen", () => {
      this.videoSource =
        this.mediaSource?.addSourceBuffer('video/mp4; codecs="avc1.640C34";') ??
        null;

      if (this.videoSource) {
        this.addListener(this.videoSource, "update", this.submit.bind(this));
      }

      this.element.play();
    });
  }

  private destroy() {
    onVideoClose();

    this.waitingForKeyFrame = true;
    this.element.pause();

    for (const { source, name, f } of this.listeners) {
      source.removeEventListener(name, f);
    }

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

  async push(data: ArrayBuffer) {
    const chunk = new Uint8Array(data);
    const headerPayloadLength = chunk[1] * 256 + chunk[0];
    const binHeader = chunk.slice(2, 2 + headerPayloadLength);

    const chunkHeader = new TextDecoder().decode(binHeader);

    onVideoChunkReceived(chunkHeader);

    const frame = chunk.slice(2 + headerPayloadLength);

    if (frame[4] === 0x66) {
      this.destroy();
      this.initialize();
      this.waitingForKeyFrame = false;
    }

    if (!this.waitingForKeyFrame) {
      this.queue.push(frame);
      this.submit();
    }
  }
}
