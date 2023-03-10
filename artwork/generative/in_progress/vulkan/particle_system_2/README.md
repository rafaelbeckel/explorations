# Particle System 2

## January 27th, 2023

For this second particle system, I want to start exploring wgpu.

The first one is mesmerizing and fun to look at, but I'm limited to about 2000 particles because it's all calculated in the CPU.

A few years ago, I did this excellent [Vulkan Tuturial](https://vulkan-tutorial.com) in C++, and I found out [there is one for Rust](https://kylemayes.github.io/vulkanalia/) as well. Let's give it a try.

I will start by following the tutorial, learn Vulkan and Vulkano first (the Vulkan library used and re-exported by Nannou) and then I will try to make something interesting using Nannou.

I might try to build something similar in Wgpu as well, to get familiar with the graphics ecosystem in Rust.

Sounds like a plan. Let's go!

### Note

After digging, I found out that the tutorial uses the lib `vulkanalia` instead of `vulkano`. It would make the boilerplate required to reach to my "Hello Triangle" much higher, so I will stick with `vulkano` and try to find another resource.

### Note 2

I went with the [official Vulkano documentation](https://vulkano.rs/guide) instead.

### Update February 13th, 2023

After following this tutorial a little bit every day, I have almost finished it. I completed the graphics pipeline part, and drew a triangle to an image. Last part now is work in the swapchain and draw it in the screen.

I feel that I'm barely scratching the surface, but at the same time this experience opens up the possibilities of what I can do.
While working in the tutorial, I have come up with a clearer vision of what I want with this particle system, expanding on the original idea.

The original idea was simply to draw about 10 million particles in the screen, and maybe make them form some image after a cool animation.

However, it would be much more exciting if we could control these particles in VR with hand gestures.

Imagine:
    - If the hands are resting, the particles slowly circle around the user's head.
    - If one hand is in a "hold" position, the particles will form a sphere on top of the hand.
    - If both hands are in a "hold" position, the particles will form a bigger sphere between both hands.
    - The user can do a "hadouken" gesture to make the particles paint a wall with a pre-defined image.
        - The image is loaded in the start of the simulation and determines the color and final position of the particles.
        - For example, a 4K image would produce about 8 million particles, each colored with a pixel of the image.
    - The first version can have a pre-defined image, but we could let the user choose any image or maybe a video.

### Update February 15th, 2023

I have finished the tutorial, and I have all the boilerplate in place and a triangle in the screen.

Next step is to start working on the particle system.

### Update February 16th, 2023

The original hello triangle tutorial has been moved to the `vulkano_tutorial` folder.

I'll clean up this project to focus in the particle system. I copied the original particle system that I'll use as a base to the references folder. It's a C# project, but simple enough so I might be able to convert it to vulkano.

The original simulates newtonian physics to attract the particles to the mouse. My implementation will be different, but for now I'll just implement it in Rust as-is.

### Update February 19th, 2023

After trying to convert the C# project to Rust, I realized I still did not build enough intuition about Vulkan and Vulkano. While Vulkano saved me some time with the boilerplate, it's hard to find resources and there is much magic going on. Doing a new hello triangle tutorial with a lower level implementation such as Ash or Vulkanalia might be beneficial because I can directly translate the more abundant C++ resources around to Rust.

Understanding Vulkan in a lower level will enable me to eventually switch to Vulkano again, but with a better understanding of what's going on.

### Update February 20th, 2023

Before starting with Vulkanalia, I'll try something quick with Vulkano and ChatGPT just for fun.

### Update February 24th, 2023

I've been stuck for a few days. I'll try to find a longer time span this weekend, so I can dig deeper. The tutorials are helpful to avoid making me think about what to do today, as I have limited time per day to update it here, and sometimes one hour is not enough to make significant progress. I kind of lost track of the Vulkano SSBO thing. I'll try to get back to it, or start a new tutorial. As I said before, a lower level understanding of Vulkan would help me a lot.

### Update February 25th, 2023

Another push into cracking vertex buffers in graphics pipelines in Vulkano. They change the API frequently, and although I'm using the latest version, their examples in the repository are on master. After figuring this out, I have locked my version to 0.32.0 and resetted my local copy of the Vulkano repository to the same version (v0.32.0 tag).

Their examples folder is really rich, and the true documentation. I'll try to allocate a vertex buffer now.

My shader works with hardcoded coordinates in the shader itself, but when I try to read the vertex buffer, the data is empty. I think I might be able to fix it now with both versions locked.

#### update 02:59

I discovered the simple_particles example, which is kind of similar to what I want to do, but it uses compute shaders to calculate the partcles positions, instead of the GPU. The other C# example I have found is calculating particle physics in the vertex_shader instead.

The plan for tomorrow is to try to implement the simple particles example with compute shaders, but with user input.

### No updates for a while (February 28th, 2023)

Yeah, that's it. What the title says. There was no significant progress on the Particle system or Vulkan in the past three days. I have made this promise to myself to update this repository every day. So this is the update for today, just in case I can't commit anything until midnight.
