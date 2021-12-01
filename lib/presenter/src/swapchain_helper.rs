//! Swapchain helper
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use crossbeam_channel::{Receiver, Sender};
use legion_graphics_api::{
    CommandBuffer, DeviceContext, Fence, Format, GfxError, GfxResult, PresentSuccessResult, Queue,
    Semaphore, Swapchain, SwapchainDef, SwapchainImage, Texture, TextureView,
};

/// May be implemented to get callbacks related to the swapchain being created/destroyed. This is
/// optional.
pub trait SwapchainEventListener {
    /// Called whenever the swapchain needs to be created (the first time, and in cases where the
    /// swapchain needs to be recreated)
    fn swapchain_created(
        &mut self,
        device_context: &DeviceContext,
        swapchain: &Swapchain,
    ) -> GfxResult<()>;

    /// Called whenever the swapchain will be destroyed (when `VkSurface` is dropped, and also in cases
    /// where the swapchain needs to be recreated)
    fn swapchain_destroyed(
        &mut self,
        device_context: &DeviceContext,
        swapchain: &Swapchain,
    ) -> GfxResult<()>;
}

// This is shared state held within an Arc between the SwapchainHelper and the PresentableFrame.
// It contains the swapchain, sync primitives required to wait for the GPU to complete work, and
// sync primitives to allow the helper/presentable frame to communicate.
struct SwapchainHelperSharedState {
    sync_frame_index: AtomicUsize,
    image_available_semaphores: Vec<Semaphore>,
    render_finished_semaphores: Vec<Semaphore>,
    in_flight_fences: Vec<Fence>,
    result_tx: Sender<GfxResult<PresentSuccessResult>>,
    result_rx: Receiver<GfxResult<PresentSuccessResult>>,
    // Arc so that we can move the swapchain to a new SwapchainHelperSharedState
    swapchain: Arc<Mutex<Swapchain>>,
}

impl SwapchainHelperSharedState {
    fn new(device_context: &DeviceContext, swapchain: Arc<Mutex<Swapchain>>) -> GfxResult<Self> {
        let image_count = swapchain.lock().unwrap().image_count();
        let mut image_available_semaphores = Vec::with_capacity(image_count);
        let mut render_finished_semaphores = Vec::with_capacity(image_count);
        let mut in_flight_fences = Vec::with_capacity(image_count);

        for _ in 0..image_count {
            image_available_semaphores.push(device_context.create_semaphore());
            render_finished_semaphores.push(device_context.create_semaphore());
            in_flight_fences.push(device_context.create_fence()?);
        }

        let (result_tx, result_rx) = crossbeam_channel::unbounded();

        Ok(Self {
            sync_frame_index: AtomicUsize::new(0),
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            result_tx,
            result_rx,
            swapchain,
        })
    }
}

/// Represents an acquired image from a swapchain. It can move between threads and be submitted by
/// any thread. The swapchain helper will not provide another image until this one is presented.
///
/// To ease error handling, the swapchain may be submitted with an error. This error will be
/// returned on the next attempt to acquire a swapchain image (i.e. the main thread).
pub struct PresentableFrame {
    // State that's shared among the swapchain helper and the presentable frame. Mostly immutable,
    // but the swapchain itself is stored in it, wrapped by a mutex
    shared_state: Option<Arc<SwapchainHelperSharedState>>,
    swapchain_image: SwapchainImage,
    sync_frame_index: usize,
}

impl PresentableFrame {
    /// An index that starts at 0 on the first present and increments every frame, wrapping back to
    /// 0 after each swapchain image has been presented once. (See `image_count` on
    /// `SwapchainHelper`). WARNING: This is not always the returned swapchain image. Swapchain
    /// images may be acquired in any order.
    pub fn rotating_frame_index(&self) -> usize {
        // The sync_frame_index can be used as-is for this purpose
        self.sync_frame_index
    }

    /// Returns the acquired swapchain image
    pub fn swapchain_texture(&self) -> &Texture {
        &self.swapchain_image.texture
    }

    /// Returns the acquired swapchain image    
    #[allow(dead_code)]
    pub fn swapchain_rtv(&self) -> &TextureView {
        &self.swapchain_image.render_target_view
    }

    /// Submits the given command buffers and schedules the swapchain image to be presented after
    /// their completion
    pub fn present(
        mut self,
        queue: &Queue,
        wait_sem: Semaphore,
        command_buffers: &[&CommandBuffer],
    ) -> GfxResult<PresentSuccessResult> {
        log::trace!(
            "Calling PresentableFrame::present with {} command buffers",
            command_buffers.len()
        );
        let result = self.do_present(queue, wait_sem, command_buffers);

        // Let the shared state arc drop, this will unblock the next frame
        let shared_state = self.shared_state.take().unwrap();
        shared_state.result_tx.send(result.clone()).unwrap();

        result
    }

    /// Present the current swapchain    
    pub fn do_present(
        &mut self,
        queue: &Queue,
        wait_sem: Semaphore,
        command_buffers: &[&CommandBuffer],
    ) -> GfxResult<PresentSuccessResult> {
        // A present can only occur using the result from the previous acquire_next_image call
        let shared_state = self.shared_state.as_ref().unwrap();
        let sync_frame_index = shared_state.sync_frame_index.load(Ordering::Relaxed);
        assert!(self.sync_frame_index == sync_frame_index);

        let frame_fence = &shared_state.in_flight_fences[sync_frame_index];
        let wait_semaphores = [
            wait_sem,
            shared_state.image_available_semaphores[sync_frame_index],
        ];
        let signal_semaphores = [shared_state.render_finished_semaphores[sync_frame_index]];

        queue.submit(
            command_buffers,
            &wait_semaphores,
            &signal_semaphores,
            Some(frame_fence),
        )?;

        let swapchain = shared_state.swapchain.lock().unwrap();

        let result = queue.present(
            &*swapchain,
            &signal_semaphores,
            self.swapchain_image.swapchain_image_index,
        )?;

        shared_state.sync_frame_index.store(
            (sync_frame_index + 1) % shared_state.in_flight_fences.len(),
            Ordering::Relaxed,
        );

        Ok(result)
    }
}

impl Drop for PresentableFrame {
    fn drop(&mut self) {
        if self.shared_state.is_some() {
            self.shared_state.take().unwrap().result_tx.send(Err(GfxError::StringError("SwapchainHelperPresentableFrame was dropped without calling present or present_with_error".to_string()))).unwrap();
        }
    }
}

/// Result of image acquisition
pub enum TryAcquireNextImageResult {
    /// Successfully retrieves a presentable frame
    Success(PresentableFrame),

    /// While this is an "error" being returned as success, it is expected and recoverable while
    /// other errors usually aren't. This way the ? operator can still be used to bail out the
    /// unrecoverable errors and the different flavors of "success" should be explicitly handled
    /// in a match
    DeviceReset,
}

/// Swap chain helper
pub struct SwapchainHelper {
    device_context: DeviceContext,
    shared_state: Option<Arc<SwapchainHelperSharedState>>,
    format: Format,
    swapchain_def: SwapchainDef,
    image_count: usize,

    // False initially, set to true when we produce the first presentable frame to indicate that
    // future frames need to wait for its result to be sent via the result_tx/result_rx channel
    expect_result_from_previous_frame: bool,
}

impl SwapchainHelper {
    /// New swapchain helper
    pub fn new(
        device_context: &DeviceContext,
        swapchain: Swapchain,
        mut event_listener: Option<&mut dyn SwapchainEventListener>,
    ) -> GfxResult<Self> {
        let format = swapchain.format();
        let image_count = swapchain.image_count();
        let swapchain_def = swapchain.swapchain_def().clone();

        let shared_state = Arc::new(SwapchainHelperSharedState::new(
            device_context,
            Arc::new(Mutex::new(swapchain)),
        )?);

        if let Some(event_listener) = event_listener.as_mut() {
            let swapchain = shared_state.swapchain.lock().unwrap();
            event_listener.swapchain_created(device_context, &*swapchain)?;
        }

        Ok(Self {
            device_context: device_context.clone(),
            shared_state: Some(shared_state),
            format,
            image_count,
            swapchain_def,
            expect_result_from_previous_frame: false,
        })
    }

    /// destroy swapchain helper
    pub fn destroy(
        &mut self,
        mut event_listener: Option<&mut dyn SwapchainEventListener>,
    ) -> GfxResult<()> {
        log::debug!("Destroying swapchain helper");

        // If there is a frame in flight, wait until it is submitted. This hopefully means we are
        // the only holder of this arc and we can unwrap it
        self.wait_until_previous_frame_submitted()?;

        if let Some(shared_state) = self.shared_state.take() {
            let begin_wait_time = instant::Instant::now();
            while Arc::strong_count(&shared_state) > 1 {
                // It's possible the previous frame has not finished dropping. Wait until this
                // occurs.
                if (instant::Instant::now() - begin_wait_time).as_secs_f32() > 1.0 {
                    // Bail early, we won't properly clean up
                    log::error!("A presentable frame was submitted but still isn't dropped. Can't clean up the swapchain");
                    break;
                }
            }

            match Arc::try_unwrap(shared_state) {
                Ok(shared_state) => {
                    log::debug!("wait for all fences to complete");
                    let fences: Vec<_> = shared_state.in_flight_fences.iter().collect();
                    self.device_context.wait_for_fences(&fences)?;

                    if let Some(event_listener) = event_listener.as_mut() {
                        let old_swapchain = shared_state.swapchain.lock().unwrap();
                        log::debug!("destroy the swapchain");
                        event_listener
                            .swapchain_destroyed(&self.device_context, &*old_swapchain)?;
                    }
                }
                Err(_arc) => {
                    let error = "The swapchain could not be destroyed, a PresentableFrame exists that is using it";
                    log::error!("{}", error);
                    return Err(error.into());
                }
            }
        }

        Ok(())
    }

    /// Get format
    #[allow(dead_code)]
    pub fn format(&self) -> Format {
        self.format
    }

    /// Get image count
    pub fn image_count(&self) -> usize {
        self.image_count
    }

    /// Get swapchain definition
    #[allow(dead_code)]
    pub fn swapchain_def(&self) -> &SwapchainDef {
        &self.swapchain_def
    }

    /// Wait until previous frame submitted
    pub fn wait_until_previous_frame_submitted(
        &mut self,
    ) -> GfxResult<Option<PresentSuccessResult>> {
        if self.expect_result_from_previous_frame {
            self.expect_result_from_previous_frame = false;

            Ok(Some(
                self.shared_state
                    .as_ref()
                    .unwrap()
                    .result_rx
                    .recv()
                    .unwrap()?,
            ))
        } else {
            Ok(None)
        }
    }

    /// wait until sync frame idle
    pub fn wait_until_sync_frame_idle(&mut self, sync_frame_index: usize) -> GfxResult<()> {
        self.shared_state.as_ref().unwrap().in_flight_fences[sync_frame_index].wait()
    }

    /// acquire next swap chain image
    pub fn acquire_next_image(
        &mut self,
        window_width: u32,
        window_height: u32,
        event_listener: Option<&mut dyn SwapchainEventListener>,
    ) -> GfxResult<PresentableFrame> {
        //
        // Block until the previous frame completes being submitted to GPU
        //
        let previous_frame_result = self.wait_until_previous_frame_submitted();

        //
        // Block until the next sync frame index finishes submitting. It's not safe to modify
        // resources associated with it until the last execution of it fully completes.
        //
        let next_sync_frame = self
            .shared_state
            .as_ref()
            .unwrap()
            .sync_frame_index
            .load(Ordering::Relaxed);
        self.wait_until_sync_frame_idle(next_sync_frame)?;

        //
        // Check the result of the previous frame. Possible outcomes:
        //  - Previous frame was successful: immediately try rendering again with the same swapchain
        //  - We've never tried rendering before: try rendering with the initial swapchain
        //  - Previous frame failed but resolvable by rebuilding the swapchain - skip trying to
        //    render again with the same swapchain
        //  - Previous frame failed with unrecoverable error: bail
        //
        let rebuild_swapchain = match &previous_frame_result {
            Ok(result) => {
                match result {
                    // We tried to render, check the previous render result
                    Some(result) => match result {
                        PresentSuccessResult::Success => false,
                        PresentSuccessResult::SuccessSuboptimal => {
                            log::debug!("Swapchain is sub-optimal, rebuilding");
                            //TODO: This can occur persistently when the app is minimized, so ignore
                            // if the size has not changed. However, we could also consider adding
                            // a counter to limit the frequency. (A sensible case for this is
                            // resizing a window - to avoid rebuilding swapchain every frame during
                            // the resize.
                            window_height != self.swapchain_def.height
                                || window_width != self.swapchain_def.width
                        }
                        PresentSuccessResult::DeviceReset => {
                            log::debug!("Swapchain sent DeviceReset, rebuilding");
                            true
                        }
                    },
                    // We have not rendered yet, so assume the swapchain we have is fine
                    None => false,
                }
            }
            // An unrecoverable failure occurred, bail
            Err(e) => return Err(e.clone()),
        };

        //
        // If we don't have any reason yet to rebuild the swapchain, try to render
        //
        if !rebuild_swapchain {
            // This case is taken if we have never rendered a frame or if the previous render was successful
            let result = self.try_acquire_next_image(window_width, window_height)?;
            if let TryAcquireNextImageResult::Success(presentable_frame) = result {
                return Ok(presentable_frame);
            }

            // if not successful (TryAcquireNextImageResult::DeviceReset), fall through to
            // try to recreate the swapchain
        };

        //
        // Rebuild the swapchain and try again. Any failure after a rebuild will be fatal
        //
        self.rebuild_swapchain(window_width, window_height, event_listener)?;

        let result = self.try_acquire_next_image(window_width, window_height)?;
        if let TryAcquireNextImageResult::Success(presentable_frame) = result {
            Ok(presentable_frame)
        } else {
            Err(GfxError::StringError(
                "Failed to recreate swapchain".to_string(),
            ))
        }
    }

    /// Try aquire next image
    pub fn try_acquire_next_image(
        &mut self,
        window_width: u32,
        window_height: u32,
    ) -> GfxResult<TryAcquireNextImageResult> {
        match self.do_try_acquire_next_image(window_width, window_height) {
            #[cfg(feature = "vulkan")]
            Err(GfxError::VkError(ash::vk::Result::ERROR_OUT_OF_DATE_KHR)) => {
                Ok(TryAcquireNextImageResult::DeviceReset)
            }
            result => result,
        }
    }

    fn do_try_acquire_next_image(
        &mut self,
        window_width: u32,
        window_height: u32,
    ) -> GfxResult<TryAcquireNextImageResult> {
        // If a frame is still outstanding from a previous acquire_next_swapchain_image call, wait
        // to receive the result of that frame. If the result was an error, return that error now.
        // This allows us to handle errors from the render thread in the main thread. This wait is
        // only blocking on getting the previous frame submitted. It's possible the GPU is still
        // processing it, and even the frame before it.
        self.wait_until_previous_frame_submitted()?;

        // check if window size changed and we are out of date
        let shared_state = self.shared_state.as_ref().unwrap();
        let mut swapchain = shared_state.swapchain.lock().unwrap();
        let swapchain_def = swapchain.swapchain_def();

        if swapchain_def.width != window_width || swapchain_def.height != window_height {
            log::debug!("Force swapchain rebuild due to changed window size");
            return Ok(TryAcquireNextImageResult::DeviceReset);
        }

        // This index iterates from 0..max_num_frames, wrapping around to 0. This ensures we use a
        // different set of sync primitives per frame in flight
        let sync_frame_index = shared_state.sync_frame_index.load(Ordering::Relaxed);

        // If this swapchain image is still being process on the GPU, block until it is flushed
        let frame_fence = &shared_state.in_flight_fences[sync_frame_index];
        self.device_context.wait_for_fences(&[frame_fence]).unwrap();

        // Acquire the next image and signal the image available semaphore when it's ready to use
        let image_available_semaphore = shared_state.image_available_semaphores[sync_frame_index];
        let swapchain_image = swapchain.acquire_next_image_semaphore(image_available_semaphore)?;

        self.expect_result_from_previous_frame = true;

        Ok(TryAcquireNextImageResult::Success(PresentableFrame {
            shared_state: Some(shared_state.clone()),
            swapchain_image,
            sync_frame_index,
        }))
    }

    fn rebuild_swapchain(
        &mut self,
        window_width: u32,
        window_height: u32,
        mut event_listener: Option<&mut dyn SwapchainEventListener>,
    ) -> GfxResult<()> {
        log::info!("Rebuild Swapchain");

        let shared_state = self.shared_state.take().unwrap();
        {
            let mut swapchain = shared_state.swapchain.lock().unwrap();
            if let Some(event_listener) = event_listener.as_mut() {
                event_listener.swapchain_destroyed(&self.device_context, &*swapchain)?;
            }

            let mut swapchain_def = swapchain.swapchain_def().clone();
            swapchain_def.width = window_width;
            swapchain_def.height = window_height;

            swapchain.rebuild(&swapchain_def)?;

            if let Some(event_listener) = event_listener.as_mut() {
                event_listener.swapchain_created(&self.device_context, &swapchain)?;
            }

            self.format = swapchain.format();
            self.image_count = swapchain.image_count();
            self.swapchain_def = swapchain_def;
        }

        self.shared_state = Some(Arc::new(SwapchainHelperSharedState::new(
            &self.device_context,
            shared_state.swapchain.clone(),
        )?));
        Ok(())
    }
}

impl Drop for SwapchainHelper {
    fn drop(&mut self) {
        // This will be a no-op if destroy() was already called
        self.destroy(None).unwrap();
    }
}
