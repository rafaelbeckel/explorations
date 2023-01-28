use bytemuck::{Pod, Zeroable};
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::device::{Device, DeviceCreateInfo, QueueCreateInfo};
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::memory::allocator::StandardMemoryAllocator;
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

    // Create the Vulkan instance with default values
    let instance =
        Instance::new(library, InstanceCreateInfo::default()).expect("failed to create instance");

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
}
