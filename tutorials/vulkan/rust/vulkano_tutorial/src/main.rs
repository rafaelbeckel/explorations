use bytemuck::{Pod, Zeroable};
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::allocator::{
    StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo,
};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferInfo};
use vulkano::device::{Device, DeviceCreateInfo, QueueCreateInfo};
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::sync::{self, GpuFuture};
use vulkano::VulkanLibrary;

// Struct to initialize the Vulkan instance
// We use the default in our example, but it's possible to customize it
//
// pub struct InstanceCreateInfo {
//     pub application_name: Option<String>,
//     pub application_version: Version,
//     pub enabled_extensions: InstanceExtensions,
//     pub enabled_layers: Vec<String>,
//     pub engine_name: Option<String>,
//     pub engine_version: Version,
//     pub max_api_version: Option<Version>,
//     pub enumerate_portability: bool,
//     pub _ne: NonExhaustive,
// }

// here we derive all these traits to ensure the data behaves as simple as possible
#[repr(C)]
#[derive(Default, Copy, Clone, Zeroable, Pod)]
struct MyStruct {
    a: u32,
    b: u32,
}

fn main() {
    let library = VulkanLibrary::new().expect("no local Vulkan library");

    // let extensions = InstanceExtensions {
    //     khr_surface: true,
    //     khr_xlib_surface: true,
    //     ..Default::default()
    // };

    // Create the Vulkan instance with default values
    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            enumerate_portability: true, // Necessary for MacOS
            ..Default::default()
        },
    )
    .expect("failed to create instance");

    // The machine may have multiple devices that support Vulkan. Here we just get the first one from our instance.
    let physical = instance
        .enumerate_physical_devices()
        .expect("could not enumerate devices")
        .next()
        .expect("no devices available");

    // Devices:
    // Initialization isn't finished yet.
    // Before being able to do anything, we have to create a device.
    //
    // A device is an object that represents an open channel of communication with a physical device,
    // and it is probably the most important object of the Vulkan API.

    // Queues and Queue Families:
    // Just like how it's possible to use multiple threads in your program running on the CPU,
    // it's also possible to run multiple operations in parallel on the GPU of your graphics card.
    // The Vulkan equivalent of a CPU thread is a queue. Queues are grouped by queue families.

    // Enumerates the queue family from our physical device:
    for family in physical.queue_family_properties() {
        println!(
            "Found a queue family with {:?} queue(s)",
            family.queue_count
        );
    }

    // We can now find the index of the first queue family that supports graphics.
    // Queue families can be for example for graphics, compute, transfer, sparse binding, etc.
    // Learn more about Queue Families in http://vulkan.gpuinfo.org
    let queue_family_index = physical
        .queue_family_properties()
        .iter()
        .enumerate()
        .position(|(_, q)| q.queue_flags.graphics)
        .expect("couldn't find a graphical queue family") as u32;

    // Creating a device returns two things: the device itself,
    // and also a list of queue objects that will later allow us to submit operations.
    let (device, mut queues) = Device::new(
        physical,
        DeviceCreateInfo {
            // here we pass the desired queue family to use by index
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            ..Default::default()
        },
    )
    .expect("failed to create device");

    let queue = queues.next().unwrap();

    // Creating a buffer _____________________________________________________________
    // When using Vulkan, you will very often need the GPU to read or write data in memory.
    // In fact there isn't much point in using the GPU otherwise, as there is nothing you can do
    // with the results of its work except write them to memory.
    //
    // In order for the GPU to be able to access some data (either for reading, writing or both),
    // we first need to create a buffer object and put the data in it.

    // Several kinds of buffers _______________________________________________________
    // Vulkano does not provide a generic Buffer struct which you could create with Buffer::new.
    // Instead, it provides several different structs that all represent buffers, each of these
    // structs being optimized for a certain kind of usage. For example, if you want to
    // continuously upload data to the GPU, you should use a CpuBufferPool.
    //
    // On the other hand, if you have some data that will largely remain visible only to the GPU,
    // a DeviceLocalBuffer brings increased performance at the cost of more complicated data access from the CPU.

    // This line differs from the tutorial.
    // I'm using Vulkano 0.32 instead of 0.31 and the API has changed.
    // The buffer creation do not take the device directly anymore. See v0.32 changelog files.
    let memory_allocator = StandardMemoryAllocator::new_default(device.clone());

    // The most simple kind of buffer that exists is the CpuAccessibleBuffer, which can be created like this:

    let _data: i32 = 12; // The content of the buffer. In this example, it's just an integer with the value 12.
    let data = MyStruct { a: 5, b: 69 }; // The buffer can be anything, though. Let's shadow the previous variable.

    let data_buffer = CpuAccessibleBuffer::from_data(
        //
        // The memory allocator for the device to use.
        // Most Vulkan objects require a reference to the device.
        &memory_allocator,
        //
        // Indicates which purpose we are creating the buffer for. This is for optimization purposes.
        // Using a buffer for a purpose that wasn't indicated in its constructor will result in an error.
        // For the sake of the example, we just create a BufferUsage that supports being used as a uniform buffer
        BufferUsage {
            uniform_buffer: true,
            ..Default::default()
        },
        //
        // Indicates if the buffer should be CPU cached. False for most cases.
        // Set to true if the application is writing data to the GPU through this buffer continuously.
        false,
        //
        // The content of the buffer.
        data,
    )
    .expect("failed to create buffer");

    // From_data and from_iter _______________________________________________________
    // In the example above we create a buffer that contains the value 12, which is of type i32.
    // But you can put any type you want in a buffer, there is no restriction.
    //
    // In order to give our arbitrary types a representation that can be used in a generic way,
    // we use the crate bytemuck and its "plain old data" trait, Pod.
    // Thus, we add the following dependency to our Cargo.toml:
    // bytemuck = "1.0"
    //
    // While it is sometimes useful to use a buffer that contains a single struct,
    // in practice it is very common to put an array of values inside of a buffer.
    // You can, for example, put an array of fifty i32s in a buffer with the
    // CpuAccessibleBuffer::from_data function.
    //
    // However, in practice, it is also very common to not know the size of the
    // array at compile-time. In order to handle this, CpuAccessibleBuffer provides
    // a from_iter constructor that takes an iterator to the data as the last parameter,
    // instead of the data itself.
    //
    // In the example below, we create a buffer that contains the value 5 of type u8, 128 times.
    // The type of the content of the buffer is [u8], which, in Rust, represents an array of u8s
    // whose size is only known at runtime.
    let iter = (0..128).map(|_| 5u8);
    let iter_buffer = CpuAccessibleBuffer::from_iter(
        &memory_allocator,
        BufferUsage {
            uniform_buffer: true,
            ..Default::default()
        },
        false,
        iter,
    )
    .unwrap();

    // Reading and writing the contents of a buffer __________________________________
    // Once a CpuAccessibleBuffer is created, you can access its content with the read() or write() methods.
    // Using read() will grant you shared access to the content of the buffer, and
    // using write() will grant you exclusive access. This is similar to using a RwLock.

    // For example, if buffer contains a MyStruct (see above):
    let mut content = data_buffer.write().unwrap();
    content.a *= 2; // `content` implements `DerefMut` whose target is of type `MyStruct` (the content of the buffer)
    content.b = 9;

    // Alternatively, suppose that the content of buffer is of type [u8] (like with the example that uses from_iter):
    let mut content = iter_buffer.write().unwrap();
    content[12] = 83; // this time `content` derefs to `[u8]`
    content[7] = 3;

    // Example operation ____________________________________________________________

    // Now that we are familiar with devices, queues, and buffers,
    // we are going to see how to ask the GPU to actually do something.

    // What we are going to ask in this example is very simple:
    // we will ask it to copy data from one buffer to another.

    // Note: You can find the full source code of this chapter here:
    // https://github.com/vulkano-rs/vulkano-www/blob/master/chapter_code/src/bin/buffer_creation.rs

    let source_content: Vec<i32> = (0..64).collect(); // array with integers from 0 to 63
    let source = CpuAccessibleBuffer::from_iter(
        &memory_allocator,
        BufferUsage {
            transfer_src: true,
            ..Default::default()
        },
        false,
        source_content,
    )
    .expect("failed to create source buffer");

    let destination_content: Vec<i32> = (0..64).map(|_| 0).collect(); // array of 64 zeros
    let destination = CpuAccessibleBuffer::from_iter(
        &memory_allocator,
        BufferUsage {
            transfer_dst: true,
            ..Default::default()
        },
        false,
        destination_content,
    )
    .expect("failed to create destination buffer");

    // Command buffers ______________________________________________________________
    // In order to ask the GPU to perform an operation,
    // we need to create a type of object that we haven't covered yet, the command buffer.

    // With Vulkan and Vulkano you can't just execute commands one by one, as it would be too inefficient.
    // Instead, we need to build a command buffer that contains a list of commands that we want to execute.

    // You can create many command buffers and use them at different times during the program.
    // They can have different uses and can do many different things.
    // In this case, we are just going to create for the operation we are trying to achieve.

    // Vulkan supports primary and secondary command buffers.
    // Primary command buffers can be sent directly to the GPU while
    // secondary command buffers allow you to store functionality that you can
    // reuse multiple times in primary command buffers.

    // We won't cover secondary command buffers here.

    // > Note: Submitting a command to the GPU can take up to several hundred microseconds,
    // > which is why we submit as many things as we can at once. OpenGL (Vulkan's predecessor)
    // > allows you to execute commands one by one, but in reality implementations buffer commands
    // > internally into command buffers. In other words, OpenGL automatically does what Vulkan requires
    // > us to do manually. In practice, OpenGL's automatic buffering often causes more harm than good in
    // > performance-critical applications.

    // This also differs from the tutorial because of Vulkano's version
    let commandbuffer_allocator = StandardCommandBufferAllocator::new(
        device.clone(),
        StandardCommandBufferAllocatorCreateInfo::default(),
    );

    // Here we'll create a builder, add a copy command to it with copy_buffer,
    // then turn that builder into an actual command buffer with .build().
    // Like we saw in the buffers creation section, we call .clone() multiple times but we only clone Arcs.
    let mut commandbuffer_builder = AutoCommandBufferBuilder::primary(
        &commandbuffer_allocator,
        queue_family_index, // the queue family that the command buffer is going to run on
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();

    commandbuffer_builder
        .copy_buffer(CopyBufferInfo::buffers(source.clone(), destination.clone()))
        .unwrap();

    let command_buffer = commandbuffer_builder.build().unwrap();

    // One thing to notice is that the AutoCommandBufferBuilder::primary() method takes as parameter a queue family index.
    // This identifies the queue family that the command buffer is going to run on.
    // In this example we don't have much choice anyway (as we only use one queue and thus one queue family),
    // but when you design a real program you have to be aware of this requirement.

    // Submission and synchronization _______________________________________________
    // The last step is to actually send the command buffer and execute it in the GPU.
    // We can do that by synchronizing with the GPU, then executing the command buffer:
    let future = sync::now(device.clone())
        .then_execute(queue.clone(), command_buffer)
        .unwrap()
        .then_signal_fence_and_flush() // same as signal fence, and then flush
        .unwrap();

    // No function in Vulkano immediately sends an operation to the GPU (with the exception of some low-level functions).
    // Instead, sync::now() creates a new type of object called a future, that keeps alive all the resources
    // that will be used by the GPU and represents the execution in time of the actual operations.

    // The future returned by sync::now() is in a pending state and makes it possible to append the execution of other
    // command buffers and operations. Only by calling .flush() are these operations all submitted at once,
    // and they  actually start executing on the GPU.

    // Using objects like this lets us build dependencies between operations and makes it possible to make an operation
    // start only after a previous one is finished, while reducing the number of slow communication operations
    // between the CPU and the GPU.

    // Now we might be tempted to try to read the content of the destination buffer as demonstrated in the previously.
    // However, because the CPU and GPU are now executing in parallel, calling destination.read() now may sometimes
    // return an error because the buffer could still be in use by the GPU.

    // In order to read the content of destination and make sure that our copy succeeded, we need to wait until the
    // operation is complete. To do that, we need to program the GPU to send back a special signal that will make us
    // know it has finished. This kind of signal is called a fence, and it lets us know whenever the GPU has reached
    // a certain point of execution.

    // Signaling a fence returns a future object called FenceSignalFuture, that has a special method .wait():
    future.wait(None).unwrap(); // None is an optional timeout

    // Only after this is done can we safely call destination.read() and check that our copy succeeded.
    let src_content = source.read().unwrap();
    let destination_content = destination.read().unwrap();
    assert_eq!(&*src_content, &*destination_content);
}
