use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess},
    command_buffer::{
        allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo},
        AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer,
        RenderPassBeginInfo, SubpassContents,
    },
    device::{
        physical::{PhysicalDevice, PhysicalDeviceType},
        Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo,
    },
    image::{view::ImageView, ImageUsage, SwapchainImage},
    instance::{Instance, InstanceCreateInfo},
    memory::allocator::StandardMemoryAllocator,
    pipeline::{
        graphics::{
            input_assembly::InputAssemblyState,
            vertex_input::BuffersDefinition,
            viewport::{Viewport, ViewportState},
        },
        GraphicsPipeline,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    shader::ShaderModule,
    swapchain::{
        self, AcquireError, Surface, Swapchain, SwapchainCreateInfo, SwapchainCreationError,
        SwapchainPresentInfo,
    },
    sync::{self, FenceSignalFuture, FlushError, GpuFuture},
};
use vulkano_win::VkSurfaceBuild;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

#[repr(C)]
#[derive(Default, Copy, Clone, Zeroable, Pod)]
struct Vertex {
    position: [f32; 2],
}

vulkano::impl_vertex!(Vertex, position);

// The vertex shader:
mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/old/vertex.glsl"
    }
}

// The fragment shader:
mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/old/fragment.glsl"
    }
}

// Function to select the best physical device for our needs.
// In the other examples, we just picked the first one.
fn select_physical_device(
    instance: &Arc<Instance>,
    surface: &Arc<Surface>,
    device_extensions: &DeviceExtensions,
) -> (Arc<PhysicalDevice>, u32) {
    instance
        // We enumerate devices as usual, but now we filter only the ones that supports the required extensions
        .enumerate_physical_devices()
        .expect("could not enumerate devices")
        .filter(|p| p.supported_extensions().contains(&device_extensions))
        // Some devices that passed the test may not have the needed queue family(ies) to present images to the surface
        // or even support graphical operations. So, we are going to filter them too and at the same time select the
        // first queue family that is suitable:
        .filter_map(|p| {
            p.queue_family_properties()
                .iter()
                .enumerate()
                // Find the first first queue family that is suitable. If none is found, `None`
                // is returned to `filter_map`, which disqualifies this physical device.
                .position(|(i, q)| {
                    q.queue_flags.graphics && p.surface_support(i as u32, &surface).unwrap_or(false)
                })
                .map(|q| (p, q as u32))
        })
        // All the physical devices that pass the filters above are suitable for the application.
        // However, not every device is equal, some are preferred over others. Now, we assign each
        // physical device a score, and pick the device with the lowest ("best") score.
        .min_by_key(|(p, _)| match p.properties().device_type {
            PhysicalDeviceType::DiscreteGpu => 0,
            PhysicalDeviceType::IntegratedGpu => 1,
            PhysicalDeviceType::VirtualGpu => 2,
            PhysicalDeviceType::Cpu => 3,

            // Note that there exists `PhysicalDeviceType::Other`, however,
            // `PhysicalDeviceType` is a non-exhaustive enum. Thus, one should
            // match wildcard `_` to catch all unknown device types.
            _ => 4,
        })
        .expect("no device available")
}

// We moved the creation of the render pass to a function to make it easier to read.
fn get_render_pass(device: Arc<Device>, swapchain: &Arc<Swapchain>) -> Arc<RenderPass> {
    vulkano::single_pass_renderpass!(
        device.clone(),
        attachments: {
            color: {
                load: Clear,
                store: Store,
                format: swapchain.image_format(),  // Sets the format the same as the swapchain
                samples: 1,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {}
        }
    )
    .unwrap()
}

// When we only had one image, we only needed to create one framebuffer for it.
// However, we now need to create a different framebuffer for each of the images.
fn get_framebuffers(
    images: &[Arc<SwapchainImage>],
    render_pass: &Arc<RenderPass>,
) -> Vec<Arc<Framebuffer>> {
    images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone()).unwrap();
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view],
                    ..Default::default()
                },
            )
            .unwrap()
        })
        .collect::<Vec<_>>()
}

// We also need to create a new command buffer for each image.
fn get_command_buffers(
    device: &Arc<Device>,
    queue: &Arc<Queue>,
    pipeline: &Arc<GraphicsPipeline>,
    framebuffers: &Vec<Arc<Framebuffer>>,
    vertex_buffer: &Arc<CpuAccessibleBuffer<[Vertex]>>,
) -> Vec<Arc<PrimaryAutoCommandBuffer>> {
    let commandbuffer_allocator = StandardCommandBufferAllocator::new(
        device.clone(),
        StandardCommandBufferAllocatorCreateInfo::default(),
    );

    framebuffers
        .iter()
        .map(|framebuffer| {
            let mut builder = AutoCommandBufferBuilder::primary(
                &commandbuffer_allocator,
                queue.queue_family_index(),
                CommandBufferUsage::MultipleSubmit, // don't forget to write the correct buffer usage
            )
            .unwrap();

            builder
                .begin_render_pass(
                    RenderPassBeginInfo {
                        clear_values: vec![Some([0.1, 0.1, 0.1, 1.0].into())],
                        ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
                    },
                    SubpassContents::Inline,
                )
                .unwrap()
                .bind_pipeline_graphics(pipeline.clone())
                .bind_vertex_buffers(0, vertex_buffer.clone())
                .draw(vertex_buffer.len() as u32, 1, 0, 0)
                .unwrap()
                .end_render_pass()
                .unwrap();

            Arc::new(builder.build().unwrap())
        })
        .collect()
}

// We also moved the creation of the graphics pipeline to a function.
fn get_pipeline(
    device: Arc<Device>,
    vs: Arc<ShaderModule>,
    fs: Arc<ShaderModule>,
    render_pass: Arc<RenderPass>,
    viewport: Viewport,
) -> Arc<GraphicsPipeline> {
    GraphicsPipeline::start()
        .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
        .vertex_shader(vs.entry_point("main").unwrap(), ())
        .input_assembly_state(InputAssemblyState::new())
        .viewport_state(ViewportState::viewport_fixed_scissor_irrelevant([viewport]))
        // You can also set the viewport to update dynamically, with a little cost in performance.
        // .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
        .fragment_shader(fs.entry_point("main").unwrap(), ())
        .render_pass(Subpass::from(render_pass, 0).unwrap())
        .build(device)
        .unwrap()
}

fn main() {
    let library = vulkano::VulkanLibrary::new().expect("no local Vulkan library/DLL");

    // We add this to the instance to enable rendering in a Window.
    // Window management is not part of Vulkan or Vulkano, so we need to use an extension.
    let required_extensions = vulkano_win::required_extensions(&library);

    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            enumerate_portability: true, // Necessary for MacOS
            enabled_extensions: required_extensions,
            ..Default::default()
        },
    )
    .expect("failed to create instance");

    // For drawing to the window, we created an object called surface.
    // The surface is a cross-platform abstraction over the actual window object
    // that vulkano can use for rendering.
    let event_loop = EventLoop::new();
    let surface = WindowBuilder::new()
        .build_vk_surface(&event_loop, instance.clone())
        .unwrap();

    // As for the window itself, it can be retrieved from the surface like this:
    let window = surface.object().unwrap().downcast_ref::<Window>().unwrap();

    // Which you can use to manipulate and change its default properties:
    window.set_title("Hello Resizable Window with Triangle!");

    // To keep the window open, we use the event loop.
    // It is typically located at the end of the main function, so scroll down to take a look at it.

    // Swapchains

    // Since we are going to draw to a window which is ultimately on the screen, things are a bit special.
    // If you were going to write directly to the window's surface, you would introduce tearing and other strange
    // artifacts, because you would be updating an image that's already visible on a screen.

    // To ensure that only complete images are shown, Vulkan uses what is called a swapchain.
    // A swapchain is a group of one or multiple images, sometimes two images but most commonly three.
    // If you have ever heard terms such as double buffering or triple buffering,
    // it refers to having respectively two or three swapchain images.

    // The idea behind a swapchain is to draw to one of its images while another one of these images is being
    // shown on the screen. When we are done drawing we ask the swapchain to show the image we have just drawn to,
    // and in return the swapchain gives us drawing access to another of its images.

    // Checking for Swapchain support:
    let device_extensions = DeviceExtensions {
        khr_swapchain: true, // This flag indicates the physical device supports swapchains.
        ..DeviceExtensions::empty()
    };

    // Note that now we filter the physical devices instead of picking the first one (see funcion above).
    let (physical, queue_family_index) =
        select_physical_device(&instance, &surface, &device_extensions);

    let (device, mut queues) = Device::new(
        physical.clone(),
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            enabled_extensions: device_extensions, // Notice that we pass the extensions here.
            ..Default::default()
        },
    )
    .expect("failed to create device");

    let queue = queues.next().unwrap();
    let memory_allocator = StandardMemoryAllocator::new_default(device.clone());

    // Creating the Swapchain:
    // Swapchains have a lot of properties: the format and dimensions of their images, an optional transformation,
    // a presentation mode, and so on. We have to specify a value for each of these parameters when we create the
    // swapchain. Therefore, we have to query the capabilities of the surface.

    let surface_capabilities = physical
        .surface_capabilities(&surface, Default::default())
        .expect("failed to get surface capabilities");

    // Of all of these properties, we only care about some of them:
    let dimensions = window.inner_size();
    let composite_alpha = surface_capabilities
        .supported_composite_alpha
        .iter()
        .next()
        .unwrap();
    let image_format = Some(
        physical
            .surface_formats(&surface, Default::default())
            .unwrap()[0]
            .0,
    );

    // Combining everything, we can create the Swapchain:
    let (mut swapchain, images) = Swapchain::new(
        device.clone(),
        surface.clone(),
        SwapchainCreateInfo {
            // This property defines how many buffers to use in the swapchain.
            // It's good to set it at least one more than the minimal,
            // to give a bit more freedom to the image queue.
            min_image_count: surface_capabilities.min_image_count + 1,
            image_format,
            image_extent: dimensions.into(),
            image_usage: ImageUsage {
                color_attachment: true, // What the images are going to be used for
                ..Default::default()
            },
            composite_alpha,
            ..Default::default()
        },
    )
    .unwrap();

    // For more info, check the Swapchain documentation:
    // https://docs.rs/vulkano/0.32/vulkano/swapchain/index.html#swapchains

    // The triangle is the same as in the last example
    let vertex1 = Vertex {
        position: [-0.5, -0.5],
    };
    let vertex2 = Vertex {
        position: [0.0, 0.5],
    };
    let vertex3 = Vertex {
        position: [0.5, -0.25],
    };

    let render_pass = get_render_pass(device.clone(), &swapchain);
    let framebuffers = get_framebuffers(&images, &render_pass);
    let vertex_buffer = CpuAccessibleBuffer::from_iter(
        &memory_allocator,
        BufferUsage {
            vertex_buffer: true, // we know the specific use of this buffer is for storing vertices.
            ..Default::default()
        },
        false,
        vec![vertex1, vertex2, vertex3].into_iter(),
    )
    .unwrap();

    let vs = vs::load(device.clone()).expect("failed to create shader module");
    let fs = fs::load(device.clone()).expect("failed to create shader module");

    let mut viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: window.inner_size().into(), // The dimensions are now set to the size of the window.
        depth_range: 0.0..1.0,
    };

    let pipeline = get_pipeline(
        device.clone(),
        vs.clone(),
        fs.clone(),
        render_pass.clone(),
        viewport.clone(),
    );

    // If you have set your pipeline to use a dynamic viewport, don't forget to then set the viewport
    // in the command buffers, by using .set_viewport(0, [viewport.clone()]).
    let mut command_buffers =
        get_command_buffers(&device, &queue, &pipeline, &framebuffers, &vertex_buffer);

    // Because we set up a static viewport in the pipeline, we have
    // to recreate the command buffers every time the window is resized.
    let mut window_resized = false;
    let mut recreate_swapchain = false;

    let frames_in_flight = images.len();
    let mut fences: Vec<Option<Arc<FenceSignalFuture<_>>>> = vec![None; frames_in_flight];
    let mut previous_fence_index = 0;

    // The event loop allows us to handle events such as window resizing, mouse movement, etc.
    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *control_flow = ControlFlow::Exit;
        }
        Event::WindowEvent {
            event: WindowEvent::Resized(_),
            ..
        } => {
            window_resized = true;
        }
        // As stated in the winit docs, the MainEventsCleared event "will be emitted when all input events have been
        // processed and redraw processing is about to begin". This essentially enables us to write functionality for each frame.
        Event::RedrawEventsCleared => {
            if window_resized || recreate_swapchain {
                recreate_swapchain = false;

                let window = surface.object().unwrap().downcast_ref::<Window>().unwrap();
                let new_dimensions = window.inner_size();

                let (new_swapchain, new_images) = match swapchain.recreate(SwapchainCreateInfo {
                    image_extent: new_dimensions.into(), // here, "image_extend" will correspond to the window dimensions
                    ..swapchain.create_info()
                }) {
                    Ok(r) => r,
                    // This error tends to happen when the user is manually resizing the window.
                    // Simply restarting the loop is the easiest way to fix this issue.
                    Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return,
                    Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
                };
                swapchain = new_swapchain;
                let new_framebuffers = get_framebuffers(&new_images, &render_pass);

                if window_resized {
                    window_resized = false;

                    viewport.dimensions = new_dimensions.into();
                    let new_pipeline = get_pipeline(
                        device.clone(),
                        vs.clone(),
                        fs.clone(),
                        render_pass.clone(),
                        viewport.clone(),
                    );
                    command_buffers = get_command_buffers(
                        &device,
                        &queue,
                        &new_pipeline,
                        &new_framebuffers,
                        &vertex_buffer,
                    );
                }
            }

            // To start drawing, the first thing that we need to do is to acquire an image to draw:

            let (image_index, suboptimal, acquire_future) =
                // The acquire_next_image() function returns the image index on which we are allowed to draw, 
                // as well as a future representing the moment when the GPU will gain access to that image.    
                match swapchain::acquire_next_image(swapchain.clone(), None) {
                    Ok(r) => r,
                    Err(AcquireError::OutOfDate) => {
                        recreate_swapchain = true;
                        return;
                    }
                    Err(e) => panic!("Failed to acquire next image: {:?}", e),
                };

            // Sometimes the function may be suboptimal, were the swapchain image will still work, but
            // may not get properly displayed. If this happens, we will signal to recreate the swapchain:
            if suboptimal {
                recreate_swapchain = true;
            }

            // The next step is to create the future that will be submitted to the GPU:
            // let execution = sync::now(device.clone())
            //     .join(acquire_future)
            //     .then_execute(queue.clone(), command_buffers[image_index as usize].clone())
            //     .unwrap()
            //     .then_swapchain_present(
            //         queue.clone(),
            //         SwapchainPresentInfo::swapchain_image_index(swapchain.clone(), image_index),
            //     )
            //     .then_signal_fence_and_flush();

            // Like we did in the other examples, we start by synchronizing.
            // However, the command buffer can't be executed immediately, as it needs to wait for the image to
            // become available. To do that, we .join() with the other future that we got from acquire_next_image(),
            // the two representing the moment where we have synchronized and acquired the said image.

            // We can then instruct the GPU to execute our main command buffer as usual (we select it by using the image index).

            // In the end, we need to present the image to the swapchain, telling it that we have finished drawing and
            // the image is ready for display. Don't forget to add a fence and flush the future.

            // We are now doing more than just executing a command buffer, so let's do a bit of error handling:
            // match execution {
            //     Ok(future) => {
            //         future.wait(None).unwrap(); // wait for the GPU to finish
            //     }
            //     Err(FlushError::OutOfDate) => {
            //         recreate_swapchain = true;
            //     }
            //     Err(e) => {
            //         println!("Failed to flush future: {:?}", e);
            //     }
            // }

            // This commented code works, but it's not optiomal.
            // Below we have the same implementation, but using frames in flight.

            // Frames in flight: executing instructions parallel to the GPU
            // Currently, the CPU waits between frames for the GPU to finish, which is somewhat inefficient.
            // What we are going to do now is to implement the functionality of frames in flight, allowing
            // the CPU to start processing new frames while the GPU is working on older ones.

            // To do that, we need to save the created fences and reuse them later.
            // Each stored fence will correspond to a new frame that is being processed in advance.
            // You can do it with only one fence (check Vulkano's triangle example if you want to do something like that).
            // However, here we will use multiple fences (likewise multiple frames in flight), which will make easier
            // for you implement any other synchronization technique you want.

            // Because each fence belongs to a specific future, we will actually store the futures as we create them,
            // which will automatically hold each of their specific resources. We won't need to synchronize each frame,
            // as we can just join with the previous frames (as all of the operations should happen continuously, anyway).

            // Note: Here we will use fence and future somewhat interchangeably, as each fence corresponds to a future
            // and vice versa. Each time we mention a fence, think of it as a future that incorporates a fence.

            // In this example we will, for simplicity, correspond each of our fences to one image, making us able to
            // use all of the existing command buffers at the same time without worrying much about what resources are
            // used in each future. If you want something different, the key is to make sure each future use resources
            // that are not already in use (this includes images and command buffers).

            // Wait for the fence related to this image to finish.
            // Normally this would be the oldest fence, that most likely have already finished.
            if let Some(image_fence) = &fences[image_index as usize] {
                image_fence.wait(None).unwrap();
            }

            // Join with the future from the previous frame, so that we only need to synchronize if the future doesn't already exist:
            let previous_future = match fences[previous_fence_index].clone() {
                // Create a NowFuture
                None => {
                    let mut now = sync::now(device.clone());

                    // Manually frees all unused resources (which could still be there because of an error)
                    now.cleanup_finished();

                    // Stores our futures in the heap, as they may have different sizes
                    now.boxed()
                }
                // Use the existing FenceSignalFuture
                Some(fence) => fence.boxed(),
            };

            // Now that we have the previous_future, we can join and create a new one as usual:
            let future = previous_future
                .join(acquire_future)
                .then_execute(queue.clone(), command_buffers[image_index as usize].clone())
                .unwrap()
                .then_swapchain_present(
                    queue.clone(),
                    SwapchainPresentInfo::swapchain_image_index(swapchain.clone(), image_index),
                )
                .then_signal_fence_and_flush();

            // And then substitute the old (obsolete) fence in the error handling:
            fences[image_index as usize] = match future {
                Ok(value) => Some(Arc::new(value)),
                Err(FlushError::OutOfDate) => {
                    recreate_swapchain = true;
                    None
                }
                Err(e) => {
                    println!("Failed to flush future: {:?}", e);
                    None
                }
            };

            // Don't forget to set previous_fence_index for the next frame:
            previous_fence_index = image_index as usize;
        }
        _ => (),
    });
}
