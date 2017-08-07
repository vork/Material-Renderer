#![allow(dead_code)]

#[macro_use]
extern crate vulkano;
#[macro_use]
extern crate vulkano_shader_derive;
extern crate vulkano_win;

extern crate winit;
extern crate cgmath;
extern crate time;
extern crate tobj;
extern crate find_folder;
extern crate image;

mod obj_loader;
mod camera_movement;

use vulkano_win::VkSurfaceBuild;
use vulkano::sync::GpuFuture;

use camera_movement::OrbitCamera;
use cgmath::Vector2;

use std::sync::Arc;

fn main() {
    let extensions = vulkano_win::required_extensions();
    let instance = vulkano::instance::Instance::new(None, &extensions, None).expect("failed to create instance");

    let physical = vulkano::instance::PhysicalDevice::enumerate(&instance)
        .next().expect("no device available");
    println!("Using device: {} (type: {:?})", physical.name(), physical.ty());

    let mut events_loop = winit::EventsLoop::new();
    let window = winit::WindowBuilder::new().build_vk_surface(&events_loop, instance.clone()).unwrap();

    let mut dimensions = {
        let (width, height) = window.window().get_inner_size_pixels().unwrap();
        [width, height]
    };

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
    let queue = queues.next().unwrap();

    let (mut swapchain, mut images) = {
        let caps = window.surface().capabilities(physical).expect("failed to get surface capabilities");

        let usage = caps.supported_usage_flags;
        let format = caps.supported_formats[0].0;
        let alpha = caps.supported_composite_alpha.iter().next().unwrap();

        vulkano::swapchain::Swapchain::new(device.clone(), window.surface().clone(), caps.min_image_count, format, dimensions, 1,
                                           usage, &queue, vulkano::swapchain::SurfaceTransform::Identity,
                                           alpha,
                                           vulkano::swapchain::PresentMode::Fifo, true, None).expect("failed to create swapchain")
    };


    let mut depth_buffer = vulkano::image::attachment::AttachmentImage::transient(device.clone(), dimensions, vulkano::format::D16Unorm).unwrap();

    let (geometry, bounds) = obj_loader::load_model("stump.obj");

    println!("bounds are: {:?}", bounds);

    let vertex_buffer = vulkano::buffer::cpu_access::CpuAccessibleBuffer
    ::from_iter(device.clone(), vulkano::buffer::BufferUsage::all(), Some(queue.family()), geometry.vertices.iter().cloned())
        .expect("failed to create buffer");

    let normals_buffer = vulkano::buffer::cpu_access::CpuAccessibleBuffer
    ::from_iter(device.clone(), vulkano::buffer::BufferUsage::all(), Some(queue.family()), geometry.normals.iter().cloned())
        .expect("failed to create buffer");

    let index_buffer = vulkano::buffer::cpu_access::CpuAccessibleBuffer
    ::from_iter(device.clone(), vulkano::buffer::BufferUsage::all(), Some(queue.family()), geometry.indices.iter().cloned())
        .expect("failed to create buffer");

    // note: this teapot was meant for OpenGL where the origin is at the lower left
    //       instead the origin is at the upper left in vulkan, so we reverse the Y axis
    let mut proj = cgmath::perspective(cgmath::Rad(std::f32::consts::FRAC_PI_2), { dimensions[0] as f32 / dimensions[1] as f32 }, 0.01, 100.0);
    let mut view = cgmath::Matrix4::look_at(
        cgmath::Point3::new((bounds.x.1 - bounds.x.0) / 2.0, (bounds.y.1 - bounds.y.0) / 2.0, bounds.z.1 + (bounds.z.1 - bounds.z.0) / 5.0),
        cgmath::Point3::new(0.0, 0.0, 0.0),
        cgmath::Vector3::new(0.0, -1.0, 0.0));
    let scale = cgmath::Matrix4::from_scale(1.0);

    let uniform_buffer = vulkano::buffer::cpu_pool::CpuBufferPool::<vs::ty::Data>
    ::new(device.clone(), vulkano::buffer::BufferUsage::all(), Some(queue.family()));

    let vs = vs::Shader::load(device.clone()).expect("failed to create shader module");
    let fs = fs::Shader::load(device.clone()).expect("failed to create shader module");

    let renderpass = Arc::new(
        single_pass_renderpass!(device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: swapchain.format(),
                    samples: 1,
                },
                depth: {
                    load: Clear,
                    store: DontCare,
                    format: vulkano::format::Format::D16Unorm,
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {depth}
            }
        ).unwrap()
    );

    let pipeline = Arc::new(vulkano::pipeline::GraphicsPipeline::start()
        .vertex_input(vulkano::pipeline::vertex::TwoBuffersDefinition::new())
        .vertex_shader(vs.main_entry_point(), ())
        .triangle_list()
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fs.main_entry_point(), ())
        .depth_stencil_simple_depth()
        .render_pass(vulkano::framebuffer::Subpass::from(renderpass.clone(), 0).unwrap())
        .build(device.clone())
        .unwrap());
    let mut framebuffers: Option<Vec<Arc<vulkano::framebuffer::Framebuffer<_, _>>>> = None;

    let mut recreate_swapchain = false;

    let mut previous_frame = Box::new(vulkano::sync::now(device.clone())) as Box<GpuFuture>;
    let rotation_start = std::time::Instant::now();

    let mut camera: OrbitCamera<f32> = OrbitCamera::new();

    let mut mouse_coords = Vector2::new(0.0f32, 0.0f32);

    loop {
        previous_frame.cleanup_finished();

        let mut done = false;
        events_loop.poll_events(|ev| {
            match ev {
                winit::Event::WindowEvent { event, .. } => {
                    match event {
                        winit::WindowEvent::Closed => done = true,
                        winit::WindowEvent::MouseInput { state, button, .. } => match state {
                            winit::ElementState::Pressed => match button {
                                winit::MouseButton::Left => camera.rotate_start(mouse_coords),
                                winit::MouseButton::Middle => camera.zoom_start(mouse_coords),
                                winit::MouseButton::Right => camera.pan_start(mouse_coords),
                                _ => ()
                            },
                            winit::ElementState::Released => match button {
                                winit::MouseButton::Left => camera.rotate_end(),
                                winit::MouseButton::Middle => camera.zoom_end(),
                                winit::MouseButton::Right => camera.pan_end(),
                                _ => ()
                            }
                        },
                        winit::WindowEvent::MouseMoved { position: (x, y), .. } => {
                            mouse_coords.x = x as f32;
                            mouse_coords.y = y as f32;
                            camera.update(mouse_coords);
                        },
                        _ => ()
                    }
                },
                _ => ()
            }
        });

        if recreate_swapchain {
            dimensions = {
                let (new_width, new_height) = window.window().get_inner_size_pixels().unwrap();
                [new_width, new_height]
            };

            let (new_swapchain, new_images) = match swapchain.recreate_with_dimension(dimensions) {
                Ok(r) => r,
                Err(vulkano::swapchain::SwapchainCreationError::UnsupportedDimensions) => {
                    continue;
                }
                Err(err) => panic!("{:?}", err)
            };

            std::mem::replace(&mut swapchain, new_swapchain);
            std::mem::replace(&mut images, new_images);

            let new_depth_buffer = vulkano::image::attachment::AttachmentImage::transient(device.clone(), dimensions, vulkano::format::D16Unorm).unwrap();
            std::mem::replace(&mut depth_buffer, new_depth_buffer);

            framebuffers = None;

            proj = cgmath::perspective(cgmath::Rad(std::f32::consts::FRAC_PI_2), { dimensions[0] as f32 / dimensions[1] as f32 }, 0.01, 100.0);

            recreate_swapchain = false;
        }

        if framebuffers.is_none() {
            let new_framebuffers = Some(images.iter().map(|image| {
                Arc::new(vulkano::framebuffer::Framebuffer::start(renderpass.clone())
                    .add(image.clone()).unwrap()
                    .add(depth_buffer.clone()).unwrap()
                    .build().unwrap())
            }).collect::<Vec<_>>());
            std::mem::replace(&mut framebuffers, new_framebuffers);
        }

        let uniform_buffer_subbuffer = {
            let uniform_data = vs::ty::Data {
                world: camera.get_transform_mat().into(),
                view: (view * scale).into(),
                proj: proj.into(),
            };

            uniform_buffer.next(uniform_data)
        };

        let set = Arc::new(vulkano::descriptor::descriptor_set::PersistentDescriptorSet::start(pipeline.clone(), 0)
            .add_buffer(uniform_buffer_subbuffer).unwrap()
            .build().unwrap()
        );

        let (image_num, acquire_future) = match vulkano::swapchain::acquire_next_image(swapchain.clone(),
                                                                                       None) {
            Ok(r) => r,
            Err(vulkano::swapchain::AcquireError::OutOfDate) => {
                recreate_swapchain = true;
                continue;
            }
            Err(err) => panic!("{:?}", err)
        };

        let command_buffer = vulkano::command_buffer::AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family()).unwrap()
            .begin_render_pass(
                framebuffers.as_ref().unwrap()[image_num].clone(), false,
                vec![
                    [0.0, 0.0, 1.0, 1.0].into(),
                    1f32.into()
                ]).unwrap()
            .draw_indexed(
                pipeline.clone(),
                vulkano::command_buffer::DynamicState {
                    line_width: None,
                    viewports: Some(vec![vulkano::pipeline::viewport::Viewport {
                        origin: [0.0, 0.0],
                        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                        depth_range: 0.0..1.0,
                    }]),
                    scissors: None,
                },
                (vertex_buffer.clone(), normals_buffer.clone()),
                index_buffer.clone(), set.clone(), ()).unwrap()
            .end_render_pass().unwrap()
            .build().unwrap();

        let future = previous_frame.join(acquire_future)
            .then_execute(queue.clone(), command_buffer).unwrap()
            .then_swapchain_present(queue.clone(), swapchain.clone(), image_num)
            .then_signal_fence_and_flush().unwrap();
        previous_frame = Box::new(future) as Box<_>;

        if done { return; }
    }
}

mod vs {
    #[derive(VulkanoShader)]
    #[ty = "vertex"]
    #[src = "
#version 450
layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 0) out vec3 v_normal;
layout(set = 0, binding = 0) uniform Data {
    mat4 world;
    mat4 view;
    mat4 proj;
} uniforms;
void main() {
    mat4 worldview = uniforms.view * uniforms.world;
    v_normal = transpose(inverse(mat3(worldview))) * normal;
    gl_Position = uniforms.proj * worldview * vec4(position, 1.0);
}
"]
    struct Dummy;
}

mod fs {
    #[derive(VulkanoShader)]
    #[ty = "fragment"]
    #[src = "
#version 450
layout(location = 0) in vec3 v_normal;
layout(location = 0) out vec4 f_color;
const vec3 LIGHT = vec3(0.0, 0.0, 1.0);
void main() {
    float brightness = dot(normalize(v_normal), normalize(LIGHT));
    vec3 dark_color = vec3(0.6, 0.0, 0.0);
    vec3 regular_color = vec3(1.0, 0.0, 0.0);
    f_color = vec4(mix(dark_color, regular_color, brightness), 1.0);
}
"]
    struct Dummy;
}