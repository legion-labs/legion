import { onVideoChunkReceived, onVideoClose } from "../api";

function addListener(obj, name, func, ctx) {
  const newFunc = ctx ? func.bind(ctx) : func;
  obj.addEventListener(name, newFunc);

  return [obj, name, newFunc];
}

function removeListeners(listeners) {
  for (const listener of listeners)
    listener[0].removeEventListener(listener[1], listener[2]);
}

export class VideoPlayer {
  constructor(element, onFatal) {
    this.element = element;
    this.onFatal = onFatal;
    this.videoSource = null;
    this.mediaSource = null;
    this.waitingForKeyFrame = true;
    this.listeners = [];
    this.queue = [];
  }

  _submit() {
    if (
      this.queue.length > 0 &&
      this.videoSource &&
      !this.videoSource.updating
    ) {
      try {
        const frame = this.queue.shift();
        this.videoSource.appendBuffer(frame);
      } catch (error) {
        console.warn(error);
        this.destroy();
        this.onFatal();
      }
    }
  }

  _init() {
    this.mediaSource = new MediaSource();
    this.element.src = URL.createObjectURL(this.mediaSource);
    this.element.load();

    this.listeners.push(
      addListener(this.element, "error", () => {
        console.error(this.element.error.message);
      })
    );

    this.listeners.push(
      addListener(this.mediaSource, "sourceopen", () => {
        this.videoSource = this.mediaSource.addSourceBuffer(
          'video/mp4; codecs="avc1.640C34";'
        );
        this.listeners.push(
          addListener(this.videoSource, "update", this._submit, this)
        );
        this.element.play();
      })
    );
  }

  _reinit() {
    this.destroy();
    this._init();
  }

  destroy() {
    onVideoClose();

    this.waitingForKeyFrame = true;
    this.element.pause();

    removeListeners(this.listeners);
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

  push(data) {
    const chunk = new Uint8Array(data);
    const headerPayloadLen = chunk[1] * 256 + chunk[0];
    const binHeader = chunk.slice(2, 2 + headerPayloadLen);

    const chunkHeader = new TextDecoder().decode(binHeader);
    onVideoChunkReceived(chunkHeader);

    const frame = chunk.slice(2 + headerPayloadLen);

    if (frame[4] === 0x66) {
      this._reinit();
      this.waitingForKeyFrame = false;
    }

    if (!this.waitingForKeyFrame) {
      this.queue.push(frame);
      this._submit();
    }
  }
}
