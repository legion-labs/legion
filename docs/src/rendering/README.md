# Rendering

> **Disclaimer**: This document is work in progress and describes our aspirations. There are technical challenges to sort out so the final outcome might look different from the original vision described here.

This section gives the high level context for the design of Legion's renderer.

### Rendering, Presenting and Streaming

Legion has 3 leaf component that are the basis of displaying a final image on screen, those components are made to be composable to suit the different scenarios we will be targeting for our pipeline:
 * Renderer: Render Graph management and frame rendering processing
 * Streamer: Sends bits or receives bits over the network
 * Presenter: Creates a swapchain to display content on screen, or an equivalent to handle the stream

You can compose the following:
 * Streaming server: composes a renderer and a streamer : renders a given scene and streams it compressed using hardware encoders.
 * Local client: composes a renderer and a presenter, renderer doing the same work as the streaming server, but the presenter takes over to locally display each frame, this will be the case for the runtime engine when we support local platforms for runtime, but also for debugging and local workflow.
 * Native Streaming client: composes a renderer, a streamer and a presenter, streamers receives compressed bits, runs hardware decompression, ingests the result into the renderer that might add some local info draws and hands off the results to the presenter.

More information on the Streamer and Presenter can be found in the the [Streaming section](../streaming/context.md).

The rest of this section focusses on the rendering parts.

### Requirements:

#### Platform support:

* For the streaming server, **only Vulkan is supported**
* For the streaming client, we support Vulkan on platforms that supports it, or platform native Apis otherwise.
* For local clients, we support Vulkan on platforms that supports it, or platform native Apis otherwise.

#### Scalability and Latency:

* Legion renderer goal is to support AAA games, meaning high production value games which push the boundaries on a combination of visual fidelity, dynamism and scale.
* 60 FPS gaming (or more in the context of AR/VR) is becoming standard, in the context of streaming it's an important aspect of latency reduction. But beyond that reducing the latency over modern renderers is a requirement (we're going to explore the idea of not having the a CPU graphic thread)

