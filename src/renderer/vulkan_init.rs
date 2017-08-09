use vulkano;
use vulkano::image::SwapchainImage;
use vulkano::swapchain::{Capabilities, Swapchain};

use vulkano_win;
use vulkano_win::{Window, VkSurfaceBuild};
use vulkano::device::{Device, Queue};
use vulkano::instance::Instance;

use winit;

use std::sync::Arc;

pub struct VulkanInit {
    pub window: Window,
    pub dimensions: [u32; 2],
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub instance: Arc<Instance>,
    pub swapchain: Arc<Swapchain>,
    pub images: Vec<Arc<SwapchainImage>>
}

impl VulkanInit {
    pub fn init(events_loop: &winit::EventsLoop) -> Self {
        // Setup the vulkan instance
        let extensions = vulkano_win::required_extensions();
        let instance = Instance::new(None, &extensions, None).expect("failed to create instance");

        // Setup the window
        let window = winit::WindowBuilder::new().build_vk_surface(&events_loop, instance.clone()).unwrap();
        let mut dimensions = {
            let (width, height) = window.window().get_inner_size_pixels().unwrap();
            [width, height]
        };

        //Setup the device
        let physical = vulkano::instance::PhysicalDevice::enumerate(&instance)
            .next().expect("no device available");
        println!("Using device: {} (type: {:?})", physical.name(), physical.ty());

        let queue = physical.queue_families().find(|&q| q.supports_graphics() &&
            window.surface().is_supported(q).unwrap_or(false))
            .expect("couldn't find a graphical queue family");

        let device_ext = vulkano::device::DeviceExtensions {
            khr_swapchain: true,
            ..vulkano::device::DeviceExtensions::none()
        };

        let (device, mut queues) = vulkano::device::Device::new(physical, physical.supported_features(),
                                                                &device_ext, [(queue, 0.5)].iter().cloned())
            .expect("failed to create device");

        //Setup the queue
        let queue = queues.next().unwrap();

        let caps = window.surface().capabilities(physical).expect("failed to get surface capabilities");

        let (mut swapchain, mut images) = {
            let usage = caps.supported_usage_flags;

            let format = caps.supported_formats[0].0;
            let alpha = caps.supported_composite_alpha.iter().next().unwrap();

            vulkano::swapchain::Swapchain::new(device.clone(), window.surface().clone(), caps.min_image_count, format, dimensions, 1,
                                               usage, &queue, vulkano::swapchain::SurfaceTransform::Identity,
                                               alpha,
                                               vulkano::swapchain::PresentMode::Fifo, true, None).expect("failed to create swapchain")
        };

        VulkanInit {
            window: window,
            dimensions: dimensions,
            device: device,
            queue: queue,
            instance: instance.clone(),
            swapchain: swapchain,
            images: images
        }
    }
}