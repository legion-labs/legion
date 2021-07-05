# Legion Realtime HW Codec crate

Uses hardware encoding/decoding to generate a stream of frames to be consumed. This crate targets latency sensitive realtime/applications and is used by the legion streamer to implement client/server functionality.

## Support Matrix

| HW Vendor         | Encoding           | Decoding           |
|-------------------|--------------------|--------------------|
| AMD (min ver)     | :heavy_check_mark: | :heavy_check_mark: |
| Android (min ver) | :x:                | :construction:     |
| Apple (min ver)   | :x:                | :construction:     |
| Intel (min ver)   | :x:                | :heavy_check_mark: |
| Nvidia (min ver)  | :heavy_check_mark: | :heavy_check_mark: |

- :heavy_check_mark: : Supported
- :construction: : Not supported but planned or wip
- :x: : not planned
