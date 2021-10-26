use graphics_api::{CommandBuffer, CommandBufferDef, CommandPool, CommandPoolDef, DefaultApi, DeviceContext, GfxApi, Queue, SwapchainDef};
use legion_ecs::prelude::Component;
use legion_renderer::Renderer;
use legion_window::{Window, WindowId};
use raw_window_handle::HasRawWindowHandle;

use crate::swapchain_helper::SwapchainHelper;

#[derive(Component)]
pub struct PresenterWindow {    
    pub window_id : WindowId,
    swapchain_helper: SwapchainHelper<DefaultApi>,
    cmd_pools: Vec<<DefaultApi as GfxApi>::CommandPool>,
    cmd_buffers: Vec<<DefaultApi as GfxApi>::CommandBuffer>,
}

impl std::fmt::Debug for PresenterWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PresenterWindow").finish()
    }
}

impl PresenterWindow {
    pub fn from_window(
        renderer: &Renderer,        
        wnd: &Window, 
        hwnd: &dyn HasRawWindowHandle ) -> Self {        

        let device_context = renderer.device_context();
        let present_queue = renderer.graphics_queue();
        let swapchain = device_context.create_swapchain(
            hwnd,
            &SwapchainDef {
                width: wnd.physical_width(),
                height: wnd.physical_height(),
                enable_vsync: true,
            }
        ).unwrap();

        let swapchain_helper =
            SwapchainHelper::<DefaultApi>::new(device_context, swapchain, None).unwrap();        
        
        let mut cmd_pools = Vec::with_capacity(swapchain_helper.image_count());
        let mut cmd_buffers = Vec::with_capacity(swapchain_helper.image_count());
        for _ in 0..swapchain_helper.image_count() {

            let cmd_pool = present_queue.create_command_pool(
                &CommandPoolDef{ transient: true }
            ).unwrap();

            let cmd_buffer = cmd_pool.create_command_buffer(
                &CommandBufferDef{ is_secondary: false }
            ).unwrap();

            cmd_pools.push(
                cmd_pool
            );

            cmd_buffers.push( 
                cmd_buffer
            );
        }

        Self {       
            window_id: wnd.id(),     
            swapchain_helper,
            cmd_pools,
            cmd_buffers,
        }
    }

    pub fn present(&mut self, wnd: &Window, present_queue: &<DefaultApi as GfxApi>::Queue ) {

        let presentable_frame = self.swapchain_helper
            .acquire_next_image(wnd.physical_width(), wnd.physical_height(), None)
            .unwrap();

        let cmd_pool = &self.cmd_pools[ presentable_frame.rotating_frame_index() ];
        let cmd_buffer = &self.cmd_buffers[ presentable_frame.rotating_frame_index() ];

        cmd_pool.reset_command_pool().unwrap();
        cmd_buffer.begin().unwrap();
        cmd_buffer.end().unwrap();

        presentable_frame
            .present(&present_queue, &[cmd_buffer])
            .unwrap();        
    }    
}

impl Drop for PresenterWindow {
    fn drop(&mut self) {
        self.swapchain_helper.destroy(None).unwrap();                
    }
}

