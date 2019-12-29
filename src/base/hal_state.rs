// #[cfg(feature = "dex12")]
use gfx_backend_vulkan as back;
// #[cfg(feature = "metal")]
// use gfx_backend_metal as back;
// #[cfg(feature = "vulkan")]
// use gfx_backend_vulkan as back;

use core::mem::ManuallyDrop;
use gfx_hal::{
    adapter::{Adapter, PhysicalDevice},
    command::{ClearColor, ClearValue, CommandBuffer,Level},
    device::Device,
    format::{Aspects, ChannelType, Format, Swizzle},
    image::{Extent, Layout, SubresourceRange, Usage, ViewKind},
    pass::{Attachment, AttachmentLoadOp, AttachmentOps, AttachmentStoreOp, SubpassDesc},
    pool::{CommandPool, CommandPoolCreateFlags},
    pso,
    pso::{PipelineStage, Rect},
    queue::{family::QueueFamily, family::QueueGroup, Submission},
    window::{Extent2D, PresentMode, PresentationSurface, Surface, Swapchain, SwapchainConfig},
    Backend, Features, Instance,
};

use winit::Window;
const DIMS: Extent2D = Extent2D {
    width: 1024,
    height: 768,
};
pub struct HalState<B: Backend> {
    current_frame: usize,
    frames_in_flight: usize,
    in_flight_fences: Vec<B::Fence>,
    // render_finished_semaphores: Vec<<back::Backend as Backend>::Semaphore>,
    // image_available_semaphores: Vec<<back::Backend as Backend>::Semaphore>,
    // command_buffers: Vec<CommandBuffer<back::Backend, Graphics>>,
    // command_pool: ManuallyDrop<CommandPool<back::Backend>>,
    // framebuffers: Vec<<back::Backend as Backend>::Framebuffer>,
    // image_views: Vec<(<back::Backend as Backend>::ImageView)>,
    // render_pass: ManuallyDrop<<back::Backend as Backend>::RenderPass>,
    // render_area: Rect,
    // queue_group: QueueGroup<back::Backend>,
    // swapchain: ManuallyDrop<<back::Backend as Backend>::Swapchain>,
    // device: ManuallyDrop<back::Device>,
    // _adapter: Adapter<back::Backend>,
    // _surface: <back::Backend as Backend>::Surface,
    // _instance: ManuallyDrop<back::Instance>,
}

impl<B> HalState<B>
where
    B: Backend,
{
    pub fn new(window: &Window) {
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

        let image_views:Vec<_>= match backbu
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
}
