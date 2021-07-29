# Streamer

> **Disclaimer**: This document is work in progress and describes our aspirations. There are technical challenges to sort out so the final outcome might look different from the original vision described here.

Legion's supports streaming a viewport to a given endpoint. The goal of the streaming crate is to build the equivalent of an interactive `<video>` html tag (we can actually replace the client side functionality on the receiving end by using a `<video>` tag)

### Input Streaming

Main crates involved: `input`, `streamer`

* Collecting inputs from client (mouse, keyboard, gamepads)
* Sends them to the server in a timely manner (no delay)
* Using a server side representation of these inputs (same constructs either receiving events from physical devices or remote devices)

### Encoding on the server 

Main crates involved: `renderer`, `codec-api`, `streamer`

* Server performs the rendering
* Compresses the images and sens them without delay through the wire

### Decoding on the client

Main crates involved: `streamer`, `codec-api`, `presenter`

* Receives a bit stream
* Decompresses it and displays it to a window 
