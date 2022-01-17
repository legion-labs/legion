# Legion Realtime HW Codec crate

Uses hardware encoding/decoding to generate a stream of frames to be consumed. This crate targets latency sensitive realtime/applications and is used by the legion streamer to implement client/server functionality.

## Support Matrix

* Minimum support of H.265 @ 4k in low latency for encoding.
* Minimum support of H.264 @ 4k in low latency mode for decoding.

| HW Vendor | Encoding                     | Decoding                      |
|-----------|------------------------------|-------------------------------|
| AMD       | (GCN3+) :heavy_check_mark:   | (GCN1+) :heavy_check_mark:    |
| Android   | :x:                          | :construction:                |
| Apple     | :x:                          | :construction:                |
| Intel     | :x:                          | (Sandy Bridge+):construction: |
| Nvidia    | (Pascal+) :construction:     | (Maxwell+) :construction:     |

- :heavy_check_mark: : Supported
- :construction: : Not supported but planned or wip
- :x: : not planned
