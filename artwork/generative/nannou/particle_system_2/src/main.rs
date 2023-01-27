use vulkano::device::{Device, DeviceCreateInfo, Features, QueueCreateInfo};
use vulkano::instance::{Instance, InstanceCreateInfo};
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

    let devide = device.clone(); // let's continue tomorrow... :D
}
