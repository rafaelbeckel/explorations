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
