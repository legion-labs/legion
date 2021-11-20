import { GrpcWebImpl, EditorClientImpl } from "proto/editor/editor";
import {
  // InitializeStreamRequest,
  // InitializeStreamResponse,
  StreamerClientImpl,
} from "proto/streaming/streaming";

// export function bytesOut(x: unknown) {
//   return window.btoa(JSON.stringify(x));
// }

// export function bytesIn(x: string) {
//   return JSON.parse(window.atob(x));
// }

const rpc = new GrpcWebImpl("http://[::1]:50051", {});

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const editorClient = new EditorClientImpl(rpc);

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const streamerClient = new StreamerClientImpl(rpc);

// #[legion_tauri_command]
// async fn initialize_stream(
//     streamer_client: tauri::State<'_, Mutex<StreamerClient<GrpcClient>>>,
//     rtc_session_description: String,
// ) -> anyhow::Result<String> {
//     let rtc_session_description = base64::decode(rtc_session_description)?;
//     let request = tonic::Request::new(InitializeStreamRequest {
//         rtc_session_description,
//     });

//     let mut streamer_client = streamer_client.lock().await;

//     let response = match streamer_client.initialize_stream(request).await {
//         Ok(response) => Ok(response.into_inner()),
//         Err(e) => {
//             error!("Error initializing stream: {}", e);

//             Err(e)
//         }
//     }?;

//     if response.error.is_empty() {
//         Ok(base64::encode(response.rtc_session_description))
//     } else {
//         Err(anyhow::format_err!("{}", response.error))
//     }
// }

export async function initializeStream(localSessionDescription: {
  toJSON(): string;
}) {
  const response = await streamerClient.InitializeStream({
    rtcSessionDescription: new TextEncoder().encode(
      JSON.stringify(localSessionDescription.toJSON())
    ),
  });

  return new RTCSessionDescription(
    JSON.parse(new TextDecoder().decode(response.rtcSessionDescription))
  );
}

// #[tauri::command]
// fn on_receive_control_message(json_msg: &str) {
//     log::info!("received control message. msg={}", json_msg);
// }

export function onReceiveControlMessage(_jsonMsg: string) {
  return;
}

// #[tauri::command]
// fn on_send_edition_command(json_command: &str) {
//     log::info!("sending edition_command={}", json_command);
// }

export function onSendEditionCommand(_jsonCommand: string) {
  return;
}

// #[tauri::command]
// fn on_video_chunk_received(chunk_header: &str) {
//     static CHUNK_INDEX_IN_FRAME_METRIC: MetricDesc = MetricDesc {
//         name: "Chunk Index in Frame",
//         unit: "",
//     };

//     static FRAME_ID_OF_CHUNK_RECEIVED: MetricDesc = MetricDesc {
//         name: "Frame ID of chunk received",
//         unit: "",
//     };

//     match json::parse(chunk_header) {
//         Ok(header) => {
//             record_json_metric(
//                 &CHUNK_INDEX_IN_FRAME_METRIC,
//                 &header["chunk_index_in_frame"],
//             );
//             record_json_metric(&FRAME_ID_OF_CHUNK_RECEIVED, &header["frame_id"]);
//         }
//         Err(e) => {
//             log::error!("Error parsing chunk header: {}", e);
//         }
//     }
// }

export function onVideoChunkReceived(_chunkHeader: string) {
  return;
}

// #[tauri::command]
// fn on_video_close() {
//     flush_log_buffer();
//     flush_metrics_buffer();
// }

export function onVideoClose() {
  return;
}

export function debounce<This, Args extends unknown[]>(
  f: (this: This, ...args: Args) => void,
  ms: number,
  immediate = false
) {
  let timeout: ReturnType<typeof setTimeout> | null;

  return function (this: This, ...args: Args) {
    // eslint-disable-next-line @typescript-eslint/no-this-alias
    const context = this;

    const later = function () {
      timeout = null;

      if (!immediate) f.apply(context, args);
    };

    const callNow = immediate && !timeout;

    if (timeout) {
      clearTimeout(timeout);
    }

    timeout = setTimeout(later, ms);

    if (callNow) {
      f.apply(context, args);
    }
  };
}

export function retryForever<T>(f: () => Promise<T>): Promise<T> {
  return retry(-1, f);
}

export async function retry<T>(
  maxRetries: number,
  f: () => Promise<T>
): Promise<T> {
  try {
    // We eagerly consume the promise and catch if it fails
    return await f();
  } catch (error) {
    if (maxRetries == 0) {
      throw error;
    }

    if (maxRetries > 0) {
      maxRetries--;
    }

    return retry(maxRetries, f);
  }
}

type Source = MediaSource | SourceBuffer | HTMLVideoElement;

type Listener = {
  source: Source;
  name: string;
  f: (this: VideoPlayer) => void;
};

function addListener(
  source: Source,
  name: string,
  f: (this: VideoPlayer) => void,
  ctx: VideoPlayer | null = null
): Listener {
  const newF = ctx ? f.bind(ctx) : f;
  source.addEventListener(name, newF);

  return { source, name, f: newF };
}

function removeListeners(listeners: Listener[]) {
  for (const { source, name, f } of listeners)
    source.removeEventListener(name, f);
}

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

  private init() {
    this.mediaSource = new MediaSource();
    this.element.src = URL.createObjectURL(this.mediaSource);
    this.element.load();

    this.listeners.push(
      addListener(this.element, "error", () => {
        console.error(this.element.error?.message);
      })
    );

    this.listeners.push(
      addListener(this.mediaSource, "sourceopen", () => {
        this.videoSource =
          this.mediaSource?.addSourceBuffer(
            'video/mp4; codecs="avc1.640C34";'
          ) ?? null;

        if (this.videoSource) {
          this.listeners.push(
            addListener(this.videoSource, "update", this.submit, this)
          );
        }

        this.element.play();
      })
    );
  }

  private reinit() {
    this.destroy();
    this.init();
  }

  private destroy() {
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

  async push(data: Blob) {
    const chunk = new Uint8Array(await data.arrayBuffer());
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
}
