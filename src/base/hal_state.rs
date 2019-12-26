// #[cfg(feature = "dex12")]
use gfx_backend_vulkan as back;
// #[cfg(feature = "metal")]
// use gfx_backend_metal as back;
// #[cfg(feature = "vulkan")]
// use gfx_backend_vulkan as back;

use core::mem::ManuallyDrop;
use gfx_hal::{
    adapter::{Adapter, PhysicalDevice},
    command::{ClearColor, ClearValue, CommandBuffer},
    device::Device,
    format::{Aspects, ChannelType, Format, Swizzle},
    image::{Extent, Layout, SubresourceRange, Usage, ViewKind},
    pass::{Attachment, AttachmentLoadOp, AttachmentOps, AttachmentStoreOp, SubpassDesc},
    pool::{CommandPool, CommandPoolCreateFlags},
    pso::{PipelineStage, Rect},
    queue::{family::QueueGroup, Submission},
    window::{PresentMode, Swapchain, SwapchainConfig},
     Features, Instance,
};

use winit::Window;

pub struct HalState<B: gfx_hal::Backend> {
    current_frame: usize,
    frames_in_flight: usize,
    in_flight_fences: Vec<<back::Backend as Backend>::Fence>,
    render_finished_semaphores: Vec<<back::Backend as Backend>::Semaphore>,
    image_available_semaphores: Vec<<back::Backend as Backend>::Semaphore>,
    command_buffers: Vec<CommandBuffer<back::Backend, Graphics>>,
    command_pool: ManuallyDrop<CommandPool<back::Backend>>,
    framebuffers: Vec<<back::Backend as Backend>::Framebuffer>,
    image_views: Vec<(<back::Backend as Backend>::ImageView)>,
    render_pass: ManuallyDrop<<back::Backend as Backend>::RenderPass>,
    render_area: Rect,
    queue_group: QueueGroup<back::Backend>,
    swapchain: ManuallyDrop<<back::Backend as Backend>::Swapchain>,
    device: ManuallyDrop<back::Device>,
    _adapter: Adapter<back::Backend>,
    _surface: <back::Backend as Backend>::Surface,
    _instance: ManuallyDrop<back::Instance>,
}

impl<B> HalState<B>
where
    B: gfx_hal::Backend,
{
    pub fn new(window: &Window) -> Result<Self, &'static str> {
        let instance = back::Instance::create("halstateWindow", 1).unwrap();

        let mut surface = instance.create_surface(window).unwrap();

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

        let mut command_pool=unsafe{
            
        }
    }
}
