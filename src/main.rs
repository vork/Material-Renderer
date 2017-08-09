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
mod renderer;

use renderer::vulkan_init::VulkanInit;

use vulkano_win::VkSurfaceBuild;
use vulkano::sync::GpuFuture;
use vulkano::image::ImmutableImage;

use camera_movement::orbit_camera::OrbitCamera;
use camera_movement::orbit_camera::OrbitZoomCameraSettings;
use cgmath::Vector2;

use find_folder::Search;

use std::sync::Arc;

fn main() {

    let mut events_loop = winit::EventsLoop::new();

    let mut vulkan_init = VulkanInit::init(&events_loop);

    let model = obj_loader::load_model("stump.obj"); //TODO make configurable

    let mut path = Search::ParentsThenKids(3, 3).for_folder("assets").unwrap();
    let (albedo, albedo_future) = {
        let mut albedo_path = path.clone();
        albedo_path.push("Aset_wood_stump_M_okfch_4K_Albedo.jpg");
        let image = image::open(&albedo_path).unwrap().to_rgba();
        let (width, height) = (image.width(), image.height());
        let data = image.into_raw().clone();

        ImmutableImage::from_iter(
            data.iter().cloned(),
            vulkano::image::Dimensions::Dim2d { width: width, height: height},
            vulkano::format::R8G8B8A8Srgb,
            Some(vulkan_init.queue.family()),
            vulkan_init.queue.clone()
        ).unwrap()
    };

    println!("bounds are: {:?}", model.bounds);

    let vertex_buffer = vulkano::buffer::cpu_access::CpuAccessibleBuffer
    ::from_iter(vulkan_init.device.clone(), vulkano::buffer::BufferUsage::all(), Some(vulkan_init.queue.family()), model.vertices.iter().cloned())
        .expect("failed to create buffer");

    let normals_buffer = vulkano::buffer::cpu_access::CpuAccessibleBuffer
    ::from_iter(vulkan_init.device.clone(), vulkano::buffer::BufferUsage::all(), Some(vulkan_init.queue.family()), model.normals.iter().cloned())
        .expect("failed to create buffer");

    let index_buffer = vulkano::buffer::cpu_access::CpuAccessibleBuffer
    ::from_iter(vulkan_init.device.clone(), vulkano::buffer::BufferUsage::all(), Some(vulkan_init.queue.family()), model.indices.iter().cloned())
        .expect("failed to create buffer");

    // note: this teapot was meant for OpenGL where the origin is at the lower left
    //       instead the origin is at the upper left in vulkan, so we reverse the Y axis
    let mut proj = cgmath::perspective(cgmath::Rad(std::f32::consts::FRAC_PI_2), {  vulkan_init.dimensions[0] as f32 / vulkan_init.dimensions[1] as f32 }, 0.01, 100.0);
    let view = cgmath::Matrix4::look_at(
        cgmath::Point3::new(
            (model.bounds.x.1 - model.bounds.x.0) / 2.0,
            (model.bounds.y.1 - model.bounds.y.0) / 2.0,
            model.bounds.z.1 + (model.bounds.z.1 - model.bounds.z.0) / 5.0),
        cgmath::Point3::new(0.0, 0.0, 0.0),
        cgmath::Vector3::new(0.0, -1.0, 0.0));
    let scale = cgmath::Matrix4::from_scale(1.0);

    let uniform_buffer = vulkano::buffer::cpu_pool::CpuBufferPool::<vs::ty::Data>
    ::new(vulkan_init.device.clone(), vulkano::buffer::BufferUsage::all(), Some(vulkan_init.queue.family()));

    let vs = vs::Shader::load(vulkan_init.device.clone()).expect("failed to create shader module");
    let fs = fs::Shader::load(vulkan_init.device.clone()).expect("failed to create shader module");

    let mut depth_buffer = vulkano::image::attachment::AttachmentImage::transient(vulkan_init.device.clone(), vulkan_init.dimensions, vulkano::format::D16Unorm).unwrap();

    let renderpass = Arc::new(
        single_pass_renderpass!(vulkan_init.device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: vulkan_init.swapchain.format(),
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
        .build(vulkan_init.device.clone())
        .unwrap());
    let mut framebuffers: Option<Vec<Arc<vulkano::framebuffer::Framebuffer<_, _>>>> = None;

    let mut recreate_swapchain = false;

    let mut previous_frame = Box::new(vulkano::sync::now(vulkan_init.device.clone())) as Box<GpuFuture>;

    let mut camera: OrbitCamera<f32> = OrbitCamera::new(OrbitZoomCameraSettings::default());

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
            vulkan_init.dimensions = {
                let (new_width, new_height) = vulkan_init.window.window().get_inner_size_pixels().unwrap();
                [new_width, new_height]
            };

            let (new_swapchain, new_images) = match vulkan_init.swapchain.recreate_with_dimension(vulkan_init.dimensions) {
                Ok(r) => r,
                Err(vulkano::swapchain::SwapchainCreationError::UnsupportedDimensions) => {
                    continue;
                }
                Err(err) => panic!("{:?}", err)
            };

            std::mem::replace(&mut vulkan_init.swapchain, new_swapchain);
            std::mem::replace(&mut vulkan_init.images, new_images);

            let new_depth_buffer = vulkano::image::attachment::AttachmentImage::transient(vulkan_init.device.clone(), vulkan_init.dimensions, vulkano::format::D16Unorm).unwrap();
            std::mem::replace(&mut depth_buffer, new_depth_buffer);

            framebuffers = None;

            proj = cgmath::perspective(cgmath::Rad(std::f32::consts::FRAC_PI_2), { vulkan_init.dimensions[0] as f32 / vulkan_init.dimensions[1] as f32 }, 0.01, 100.0);

            recreate_swapchain = false;
        }

        if framebuffers.is_none() {
            let new_framebuffers = Some(vulkan_init.images.iter().map(|image| {
                Arc::new(vulkano::framebuffer::Framebuffer::start(renderpass.clone())
                    .add(image.clone()).unwrap()
                    .add(depth_buffer.clone()).unwrap()
                    .build().unwrap())
            }).collect::<Vec<_>>());
            std::mem::replace(&mut framebuffers, new_framebuffers);
        }

        let uniform_buffer_subbuffer = {
            let uniform_data = vs::ty::Data {
                world: camera.camera().orthogonal().into(),
                view: (view * scale).into(),
                proj: proj.into(),
            };

            uniform_buffer.next(uniform_data)
        };

        let set = Arc::new(vulkano::descriptor::descriptor_set::PersistentDescriptorSet::start(pipeline.clone(), 0)
            .add_buffer(uniform_buffer_subbuffer).unwrap()
            .build().unwrap()
        );

        let (image_num, acquire_future) = match vulkano::swapchain::acquire_next_image(vulkan_init.swapchain.clone(),
                                                                                       None) {
            Ok(r) => r,
            Err(vulkano::swapchain::AcquireError::OutOfDate) => {
                recreate_swapchain = true;
                continue;
            }
            Err(err) => panic!("{:?}", err)
        };

        let command_buffer = vulkano::command_buffer::AutoCommandBufferBuilder::primary_one_time_submit(vulkan_init.device.clone(), vulkan_init.queue.family()).unwrap()
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
                        dimensions: [vulkan_init.dimensions[0] as f32, vulkan_init.dimensions[1] as f32],
                        depth_range: 0.0..1.0,
                    }]),
                    scissors: None,
                },
                (vertex_buffer.clone(), normals_buffer.clone()),
                index_buffer.clone(), set.clone(), ()).unwrap()
            .end_render_pass().unwrap()
            .build().unwrap();

        let future = previous_frame.join(acquire_future)
            .then_execute(vulkan_init.queue.clone(), command_buffer).unwrap()
            .then_swapchain_present(vulkan_init.queue.clone(), vulkan_init.swapchain.clone(), image_num)
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