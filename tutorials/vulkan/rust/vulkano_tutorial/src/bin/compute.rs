use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::allocator::{
    StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo,
};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, QueueCreateInfo};
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::pipeline::{ComputePipeline, Pipeline, PipelineBindPoint};
use vulkano::sync::{self, GpuFuture};

fn main() {
    let library = vulkano::VulkanLibrary::new().expect("no local Vulkan library/DLL");
    // Create the Vulkan instance with default values
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
            enabled_extensions: DeviceExtensions {
                khr_storage_buffer_storage_class: true,
                ..DeviceExtensions::empty()
            },
            ..Default::default()
        },
    )
    .expect("failed to create device");

    let queue = queues.next().unwrap();
    let memory_allocator = StandardMemoryAllocator::new_default(device.clone());

    // Introduction to compute operations
    let data_iter = 0..65536;
    let data_buffer = CpuAccessibleBuffer::from_iter(
        &memory_allocator,
        BufferUsage {
            storage_buffer: true,
            ..Default::default()
        },
        false,
        data_iter,
    )
    .expect("failed to create buffer");

    // Compute pipelines
    mod cs {
        vulkano_shaders::shader! {
            ty: "compute",
            src: "
#version 450
layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;
layout(set = 0, binding = 0) buffer Data {
    uint data[];
} buf;
void main() {
    uint idx = gl_GlobalInvocationID.x;
    buf.data[idx] *= 12;
}"
        }
    }

    let shader = cs::load(device.clone()).expect("failed to create shader module");
    let compute_pipeline = ComputePipeline::new(
        device.clone(),
        shader.entry_point("main").unwrap(),
        &(),
        None,
        |_| {},
    )
    .expect("failed to create compute pipeline");

    let descriptorset_allocator = StandardDescriptorSetAllocator::new(device.clone());
    let layout = compute_pipeline.layout().set_layouts().get(0).unwrap();

    let set = PersistentDescriptorSet::new(
        &descriptorset_allocator,
        layout.clone(),
        [WriteDescriptorSet::buffer(0, data_buffer.clone())], // 0 is the binding
    )
    .unwrap();

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
        .bind_pipeline_compute(compute_pipeline.clone())
        .bind_descriptor_sets(
            PipelineBindPoint::Compute,
            compute_pipeline.layout().clone(),
            0, // 0 is the index of our set
            set,
        )
        .dispatch([1024, 1, 1])
        .unwrap();
    let command_buffer = builder.build().unwrap();

    let future = sync::now(device.clone())
        .then_execute(queue.clone(), command_buffer)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap();

    future.wait(None).unwrap();

    let content = data_buffer.read().unwrap();
    for (n, val) in content.iter().enumerate() {
        assert_eq!(*val, n as u32 * 12);
    }

    println!("Everything succeeded!");
}
