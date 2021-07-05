# Rendering - Context

This document describes assumptions related to the renderer.

### Rendering, Presenting and Streaming

Legion has 3 leaf component that are the basis of displaying a final image on screen, those components are made to be composable to suit the different scenarios we will be targeting for our pipeline:
 * Renderer: Render Graph management and frame rendering processing
 * streamer: Sends bits or receives bits over the network
 * presenter: Creates a swapchain to display content on screen
You can compose the following:
 * Streaming server: composes a renderer and a streamer : renders a given scene and streams it compressed using hardware encoders.
 * Local client: composes a renderer and a presenter, renderer doing the same work as the streaming server, but the presenter takes over to locally display each frame, this will be the case for the runtime engine when we support local platforms for runtime, but also for debugging and local workflow.
 * Streaming client: composes a renderer, a streamer and a presenter, streamers receives compressed bits, runs hardware decompression, ingests the result into the renderer that might add some local info draws and hands off the results to the presenter.

### Platform support:
* For the streaming server, **only Vulkan is supported**
* For the streaming client, we support Vulkan on platforms that supports it, or platform native Apis otherwise.
* For local clients, we support Vulkan on platforms that supports it, or platform native Apis otherwise.
