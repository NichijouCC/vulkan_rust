// #[cfg(feature = "dex12")]
use gfx_backend_vulkan as back;
// #[cfg(feature = "metal")]
// use gfx_backend_metal as back;
// #[cfg(feature = "vulkan")]
// use gfx_backend_vulkan as back;

use gfx_hal::{
    adapter::{Adapter, PhysicalDevice},
    command::{ClearColor, ClearValue, CommandBuffer, CommandBufferFlags, Level, SubpassContents},
    device::Device,
    format::{Aspects, ChannelType, Format, Swizzle},
    image::{Extent, Layout, SubresourceRange, Usage, ViewKind},
    pass::{Attachment, AttachmentLoadOp, AttachmentOps, AttachmentStoreOp, SubpassDesc},
    pool::{CommandPool, CommandPoolCreateFlags},
    pso,
    pso::{PipelineStage, Rect, Viewport},
    queue::{family::QueueFamily, family::QueueGroup, CommandQueue, Submission},
    window::{Extent2D, PresentMode, PresentationSurface, Surface, Swapchain, SwapchainConfig},
    Backend, Features, Instance,
};

use winit::Window;

use std::{
    borrow::Borrow,
    iter,
    mem::{self, ManuallyDrop},
};

const DIMS: Extent2D = Extent2D {
    width: 1024,
    height: 768,
};
pub struct HalState<B: Backend> {
    frame: usize,
    frames_in_flight: usize,
    // in_flight_fences: Vec<B::Fence>,
    // render_finished_semaphores: Vec<<back::Backend as Backend>::Semaphore>,
    // image_available_semaphores: Vec<<back::Backend as Backend>::Semaphore>,
    cmd_pools: Vec<B::CommandPool>,
    cmd_buffers: Vec<B::CommandBuffer>,
    // framebuffers: Vec<<back::Backend as Backend>::Framebuffer>,
    // image_views: Vec<(<back::Backend as Backend>::ImageView)>,
    render_pass: ManuallyDrop<B::RenderPass>,
    // render_area: Rect,
    queue_group: QueueGroup<B>,
    // swapchain: ManuallyDrop<<back::Backend as Backend>::Swapchain>,
    submission_complete_fences: Vec<B::Fence>,
    submission_complete_semaphores: Vec<B::Semaphore>,
    device: B::Device,
    adapter: Adapter<B>,
    surface: B::Surface,
    format: Format,
    dimensions: Extent2D,
    viewport: Viewport, // _instance: ManuallyDrop<back::Instance>,
}

impl<B> HalState<B>
where
    B: Backend,
{
    pub fn new(window: &Window) -> Self {
        let instance = B::Instance::create("halstateWindow", 1).unwrap();

        let mut surface = unsafe { instance.create_surface(window).unwrap() };

        let adapter: Adapter<B> = instance
            .enumerate_adapters()
            .into_iter()
            .find(|a| {
                a.queue_families.iter().any(|qf| {
                    qf.queue_type().supports_graphics() && surface.supports_queue_family(qf)
                })
            })
            .ok_or("Couldn't find a graphical Adapter!")?;

        let queue_family = adapter
            .queue_families
            .iter()
            .find(|qf| qf.queue_type().supports_graphics() && surface.supports_queue_family(qf))
            .ok_or("Couldn't find a QueueFamily with graphics!")?;

        let mut gpu = unsafe {
            adapter
                .physical_device
                .open(&[(queue_family, &[1.0])], Features::empty())
                .unwrap()
        };

        let queue_group = gpu.queue_groups.pop().unwrap();
        let device = gpu.device;

        let mut command_pool = unsafe {
            device.create_command_pool(queue_group.family, CommandPoolCreateFlags::empty())
        }
        .expect("can't create command pool");

        let caps = surface.capabilities(&adapter.physical_device);
        let formats = surface.supported_formats(&adapter.physical_device);
        let format = formats.map_or(Format::Rgba8Srgb, |formats| {
            formats
                .iter()
                .find(|format| format.base_format().1 == ChannelType::Srgb)
                .map(|format| *format)
                .unwrap_or(formats[0])
        });
        let swap_config = SwapchainConfig::from_caps(&caps, format, DIMS);
        let extent = swap_config.extent;
        unsafe {
            surface
                .configure_swapchain(&device, swap_config)
                .expect("Can't create swapchain");
        }

        let render_pass = {
            let color_attachment = Attachment {
                format: Some(format),
                samples: 1,
                ops: AttachmentOps {
                    load: AttachmentLoadOp::Clear,
                    store: AttachmentStoreOp::Store,
                },
                stencil_ops: AttachmentOps::DONT_CARE,
                layouts: Layout::Undefined..Layout::Present,
            };
            let subpass = SubpassDesc {
                colors: &[(0, Layout::ColorAttachmentOptimal)],
                depth_stencil: None,
                inputs: &[],
                resolves: &[],
                preserves: &[],
            };
            unsafe {
                device
                    .create_render_pass(&[color_attachment], &[subpass], &[])
                    .map_err(|_| "Couldn't create a render pass!")?
            }
        };

        let frames_in_flight = 3;
        // The number of the rest of the resources is based on the frames in flight.
        let mut submission_complete_semaphores = Vec::with_capacity(frames_in_flight);
        let mut submission_complete_fences = Vec::with_capacity(frames_in_flight);

        let mut cmd_pools = Vec::with_capacity(frames_in_flight);
        let mut cmd_buffers = Vec::with_capacity(frames_in_flight);

        cmd_pools.push(command_pool);
        for _ in 1..frames_in_flight {
            unsafe {
                cmd_pools.push(
                    device
                        .create_command_pool(queue_group.family, CommandPoolCreateFlags::empty())
                        .expect("Can't create command pool"),
                );
            }
        }

        for i in 0..frames_in_flight {
            submission_complete_semaphores.push(
                device
                    .create_semaphore()
                    .expect("Could not create semaphore"),
            );
            submission_complete_fences
                .push(device.create_fence(true).expect("Could not create fence"));
            cmd_buffers.push(unsafe { cmd_pools[i].allocate_one(Level::Primary) });
        }

        let viewport = pso::Viewport {
            rect: pso::Rect {
                x: 0,
                y: 0,
                w: extent.width as _,
                h: extent.height as _,
            },
            depth: 0.0..1.0,
        };

        HalState {
            frame: 0,
            frames_in_flight,
            adapter,
            surface,
            device,
            dimensions: DIMS,
            viewport,
            queue_group,
            format,
            render_pass,
            submission_complete_semaphores,
            submission_complete_fences,
            cmd_buffers,
            cmd_pools,
        }

        // let image_views:Vec<_>= match backbu
        // let set_layout=ManuallyDrop::new(
        //     unsafe{
        //         device.create_descriptor_set_layout(
        //             &[
        //                 pso::DescriptorSetLayoutBinding{
        //                     binding:0,
        //                     ty:pso::DescriptorType::
        //                 }
        //             ]
        //         )
        //     }
        // )
    }

    fn render(&mut self) {
        let surface_image = unsafe {
            match self.surface.acquire_image(!0) {
                Ok((image, _)) => image,
                Err(_) => {
                    self.create_swapchain();
                    return;
                }
            }
        };
        let framebuffer = unsafe {
            self.device
                .create_framebuffer(
                    &self.render_pass,
                    iter::once(surface_image.borrow()),
                    Extent {
                        width: self.dimensions.width,
                        height: self.dimensions.height,
                        depth: 1,
                    },
                )
                .unwrap()
        };

        let frame_idx = self.frame as usize % self.frames_in_flight;

        unsafe {
            let fence = &self.submission_complete_fences[frame_idx];
            self.device
                .wait_for_fence(fence, !0)
                .expect("Failed to wait for fence");
            self.device
                .reset_fence(fence)
                .expect("Failed to reset fence");
            self.cmd_pools[frame_idx].reset(false);
        }

        let cmd_buffer = &mut self.cmd_buffers[frame_idx];
        unsafe {
            cmd_buffer.begin_primary(CommandBufferFlags::ONE_TIME_SUBMIT);
            cmd_buffer.begin_render_pass(
                &self.render_pass,
                &framebuffer,
                self.viewport.rect,
                &[ClearValue {
                    color: ClearColor {
                        float32: [0.8, 0.8, 0.8, 1.0],
                    },
                }],
                SubpassContents::Inline,
            );
            cmd_buffer.end_render_pass();
            cmd_buffer.finish();

            let submission = Submission {
                command_buffers: iter::once(&*cmd_buffer),
                wait_semaphores: None,
                signal_semaphores: iter::once(&self.submission_complete_semaphores[frame_idx]),
            };
            self.queue_group.queues[0].submit(
                submission,
                Some(&self.submission_complete_fences[frame_idx]),
            );

            // present frame
            let result = self.queue_group.queues[0].present_surface(
                &mut self.surface,
                surface_image,
                Some(&self.submission_complete_semaphores[frame_idx]),
            );

            self.device.destroy_framebuffer(framebuffer);

            if result.is_err() {
                self.create_swapchain();
            }
        }
        self.frame += 1;
    }

    fn create_swapchain(&mut self) {
        let caps = self.surface.capabilities(&self.adapter.physical_device);
        let swap_config = SwapchainConfig::from_caps(&caps, self.format, self.dimensions);
        println!("{:?}", swap_config);
        let extent = swap_config.extent.to_extent();

        unsafe {
            self.surface
                .configure_swapchain(&self.device, swap_config)
                .expect("Can't create swapchain");
        }

        self.viewport.rect.w = extent.width as _;
        self.viewport.rect.h = extent.height as _;
    }
}
