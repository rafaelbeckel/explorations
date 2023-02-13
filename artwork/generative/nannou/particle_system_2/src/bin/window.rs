// Let's do the last part of the tutorial now.

// For now, I just copied the graphics.rs file to hold the boilerplate.
// I'll change this as needed as the tutorial goes.

use bytemuck::{Pod, Zeroable};
use image::{ImageBuffer, Rgba};
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::{
        allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo},
        AutoCommandBufferBuilder, CommandBufferUsage, CopyImageToBufferInfo, RenderPassBeginInfo,
        SubpassContents,
    },
    device::{Device, DeviceCreateInfo, QueueCreateInfo},
    format::Format,
    image::{view::ImageView, ImageDimensions, StorageImage},
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
    render_pass::{Framebuffer, FramebufferCreateInfo, Subpass},
    sync::{self, GpuFuture},
};

// The first step to describe a shape with vulkano is to create a struct named Vertex
// (the actual name doesn't matter) whose purpose is to describe the properties of a
// single vertex. Once this is done, the shape of our triangle is going to be a buffer
// whose content is an array of three Vertex objects.
#[repr(C)]
#[derive(Default, Copy, Clone, Zeroable, Pod)]
struct Vertex {
    position: [f32; 2],
}

// The impl_vertex! macro is makes it possible for vulkano to build the link between
// the content of the buffer and the input of the vertex shader.
vulkano::impl_vertex!(Vertex, position);

fn main() {
    let library = vulkano::VulkanLibrary::new().expect("no local Vulkan library/DLL");
    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            enumerate_portability: true, // Necessary for MacOS
            ..Default::default()
        },
    )
    .expect("failed to create instance");

    let physical = instance
        .enumerate_physical_devices()
        .expect("could not enumerate devices")
        .next()
        .expect("no devices available");

    let queue_family_index = physical
        .queue_family_properties()
        .iter()
        .enumerate()
        .position(|(_, q)| q.queue_flags.graphics)
        .expect("couldn't find a graphical queue family") as u32;

    let (device, mut queues) = Device::new(
        physical,
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            ..Default::default()
        },
    )
    .expect("failed to create device");

    let queue = queues.next().unwrap();
    let memory_allocator = StandardMemoryAllocator::new_default(device.clone());

    let image = StorageImage::new(
        &memory_allocator,
        ImageDimensions::Dim2d {
            width: 1024,
            height: 1024,
            array_layers: 1,
        },
        Format::R8G8B8A8_UNORM,
        Some(queue.queue_family_index()),
    )
    .unwrap();

    let buffer = CpuAccessibleBuffer::from_iter(
        &memory_allocator,
        BufferUsage {
            transfer_dst: true,
            ..Default::default()
        },
        false,
        (0..1024 * 1024 * 4).map(|_| 0u8),
    )
    .expect("failed to create buffer");

    // Let's define a simple triangle
    let vertex1 = Vertex {
        position: [-0.5, -0.5],
    };
    let vertex2 = Vertex {
        position: [0.0, 0.5],
    };
    let vertex3 = Vertex {
        position: [0.5, -0.25],
    };

    // Now all we have to do is create a buffer that contains these three vertices.
    // This buffer will be passed as a parameter when we start the drawing operation.
    // A buffer that contains a collection of vertices is commonly named a vertex buffer.
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

    // > Note: Vertex buffers are not special in any way. The term vertex buffer indicates the way
    // the programmer intends to use the buffer, and it is not a property of the buffer.

    // The vertex shader:
    mod vs {
        vulkano_shaders::shader! {
            ty: "vertex",

            // The layout definition declares that each vertex has an attribute named position and of type vec2.
            // This corresponds to the definition of the Vertex struct we created.
            // We also define a single output named out_color for the vertex color.
            src: "
                #version 450

                layout(location = 0) in vec2 position;

                layout(location = 0) out vec3 out_color;

                void main() {
                    const vec3 colors[3] = vec3[3](
                        vec3(1.0f, 0.0f, 0.0f), //red
                        vec3(0.0f, 1.0f, 0.0f), //green
                        vec3(0.0f, 0.0f, 1.0f)  //blue
                    );

                    gl_Position = vec4(position, 0.0, 1.0);
                    out_color = colors[gl_VertexIndex];
                }
            "
        }
    }

    // After the vertex shader has run on each vertex, the next step that the GPU performs
    // is to determine which pixels of the target image are within the shape of the triangle.
    // Only these pixels will be modified on the final image.

    // The fragment shader:
    mod fs {
        vulkano_shaders::shader! {
            ty: "fragment",

            // The layout definition declares a single output named f_color.
            // Vulkan gives you the possibility to draw to multiple images at once,
            // which is why we need to declare each output and its type.
            // Drawing to multiple images at once is an advanced topic that isn't covered here.
            src: "
                #version 450

                layout(location = 0) in vec3 in_color;

                layout(location = 0) out vec4 f_color;

                void main() {
                    f_color = vec4(in_color, 1.0);
                }
            "
        }
    }

    // The fragment shader runs on each pixel covered by the shape from the vertex shader.

    let vs = vs::load(device.clone()).expect("failed to create shader module");
    let fs = fs::load(device.clone()).expect("failed to create shader module");

    // Render Passes:
    // In order to fully optimize and parallelize command execution, we can't just ask the GPU to
    // draw a shape whenever we want. Instead we first have to enter a special "rendering mode" by
    // entering what is called a render pass. We can only draw when we have entered a render pass.

    // The term "render pass" describes two things:
    // - It designates the "rendering mode" we have to enter before we can add drawing commands to a command buffer.
    // - It also designates a kind of object that describes this rendering mode.

    // For the moment, the only thing we want to do is draw some color to a single image.
    // This is the most simple case possible, and we only need to provide two things to a render pass:
    // the format of the image, and the fact that we don't use multisampling (which is an anti-aliasing technique).

    // More complex games can use render passes in very complex ways, with multiple subpasses and multiple attachments,
    // and with various micro-optimizations. Vulkano's API is suitable for both the simple cases and the complex usages,
    // which is why it may look complex at first.

    let render_pass = vulkano::single_pass_renderpass!(
        device.clone(),

        // A render pass is made of attachments and passes.
        // Here we declare one attachment whose name is color (the name is arbitrary),
        // and one pass that will use color as its single output.
        attachments: {
            color: {
                load: Clear, // Indicates that we want the GPU to clear the image when entering the render pass (i.e. fill it with a single color)
                store: Store, // Indicates that we want the GPU to store the output of our draw commands to the image.
                format: Format::R8G8B8A8_UNORM,
                samples: 1,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {}
        }
    )
    .unwrap();

    // Note: It is possible to create temporary images whose content is only relevant inside of a render pass,
    // in which case it is optimal to use store: DontCare instead of store: Store.

    // Frame Buffer:
    // A render pass only describes the format and the way we load and store the image we are going to draw upon.
    // It is enough to initialize all the objects we need.

    // But before we can draw, we also need to indicate the actual list of attachments.
    // This is done by creating a framebuffer.

    // Creating a framebuffer is typically done as part of the rendering process.
    // It is not a bad idea to keep the framebuffer objects alive between frames, but it won't
    // kill your performance to create and destroy a few framebuffer objects during some frames.

    let view = ImageView::new_default(image.clone()).unwrap();
    let framebuffer = Framebuffer::new(
        render_pass.clone(),
        FramebufferCreateInfo {
            attachments: vec![view],
            ..Default::default()
        },
    )
    .unwrap();

    // We are now ready the enter drawing mode!

    // This is done by calling the begin_render_pass function on the command buffer builder.
    // This function takes as parameter the framebuffer, a enum, and a Vec that contains the colors to fill
    // the attachments with. Since we have only one single attachment, this Vec contains only one element.

    // Clearing our attachment has exactly the same effect as the clear_color_image function we covered previously,
    // except that this time it is done by the rendering engine.

    // The enum passed as second parameter describes whether we are going to directly invoke draw commands or use
    // secondary command buffers instead. Secondary command buffers are a more advanced topic.
    // Be we are using only direct commands, we will leave it as ::Inline

    // As a demonstration, let's just enter a render pass and leave it immediately after:

    /*
        let commandbuffer_allocator = StandardCommandBufferAllocator::new(
            device.clone(),
            StandardCommandBufferAllocatorCreateInfo::default(),
        );

        let mut builder = AutoCommandBufferBuilder::primary(
            &commandbuffer_allocator,
            queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.0, 0.0, 1.0, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
                },
                SubpassContents::Inline,
            )
            .unwrap()
            .end_render_pass()
            .unwrap();
    */

    // The graphics pipeline:
    // Just like we had to create a compute pipeline in order to perform a compute operation,
    // we have to create a graphics pipeline before we perform a draw operation.

    // When we draw, we have the possibility to draw only to a specific rectangle of the screen called a viewport.
    // The borders of the viewport will map to the -1.0 and 1.0 logical coordinates that we covered in the vertex
    // input section of the guide. Any part of the shape that ends up outside of this rectangle will be discarded.
    let viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [1024.0, 1024.0],
        depth_range: 0.0..1.0,
    };

    let pipeline = GraphicsPipeline::start()
        // Describes the layout of the vertex input and how should it behave
        .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
        // A Vulkan shader can in theory contain multiple entry points, so we have to specify
        // which one.
        .vertex_shader(vs.entry_point("main").unwrap(), ())
        // Indicate the type of the primitives (the default is a list of triangles)
        .input_assembly_state(InputAssemblyState::new())
        // Set the fixed viewport
        .viewport_state(ViewportState::viewport_fixed_scissor_irrelevant([viewport]))
        // Same as the vertex input, but this for the fragment input
        .fragment_shader(fs.entry_point("main").unwrap(), ())
        // This graphics pipeline object concerns the first pass of the render pass.
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        // Now that everything is specified, we call `build`.
        .build(device.clone())
        .unwrap();

    // The state ViewportState::viewport_fixed_scissor_irrelevant() configures the builder so that we use one
    // specific viewport, and that the state of this viewport is fixed. This makes it not possible to change
    // the viewport for each draw command, but adds more performance. Because we are drawing only one image
    // and not changing the viewport between draws, this is the optimal approach. If you wanted to draw to
    // another image of a different size, you would have to create a new pipeline object. Another approach
    // would be to use a dynamic viewport, where you would pass your viewport in the command buffer instead.

    // Note: If you configure multiple viewports, you can use geometry shaders to choose which viewport the
    // shape is going to be drawn to. This topic isn't covered here.

    // Drawing:
    // Now that we have all the ingredients, it is time to bind everything and insert a draw call inside of our render pass.
    // To draw the triangle, we need to pass the pipeline, the vertex_buffer and the actual draw command:

    let commandbuffer_allocator = StandardCommandBufferAllocator::new(
        device.clone(),
        StandardCommandBufferAllocatorCreateInfo::default(),
    );

    let mut builder = AutoCommandBufferBuilder::primary(
        &commandbuffer_allocator,
        queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
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
        .draw(
            3, 1, 0, 0, // 3 is the number of vertices, 1 is the number of instances
        )
        .unwrap()
        .end_render_pass()
        .unwrap()
        .copy_image_to_buffer(CopyImageToBufferInfo::image_buffer(image, buffer.clone()))
        .unwrap();

    let command_buffer = builder.build().unwrap();

    let future = sync::now(device.clone())
        .then_execute(queue.clone(), command_buffer)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap();
    future.wait(None).unwrap();

    let buffer_content = buffer.read().unwrap();
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(1024, 1024, &buffer_content[..]).unwrap();
    image.save("hello_triangle.png").unwrap();

    println!("Everything succeeded!");
}
