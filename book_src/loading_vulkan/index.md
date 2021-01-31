
# Loading Vulkan

The [Vulkan](https://www.khronos.org/vulkan/) API is the *new hotness* in graphics APIs.
You'll also see it called "Vk" for short.

Parts of it are really cool,
but it's also verbose as heck to use.
Like *really* verbose.
Eventually you do get performance and control gains compared to using something like GL,
but the initial setup is way higher than with the previous era of drawing APIs.
The common estimate is that it's 1000 lines to draw a triangle in Vulkan,
not even counting any of the support libraries commonly used.

Despite all that,
*initializing* Vulkan is actually far easier than initializing OpenGL.
It's practically just one OS call,
grab some function pointers,
and then an API call or two and you're off.

One thing to note is that the core of Vulkan is *smaller* than the core of OpenGL,
and even drawing images to the screen is considered an "optional" part of a Vulkan implementation.
The most minimal thing to do in Vulkan is rendering into memory,
so we'll start by doing that as a "test run" of sorts.
After we've done a test render into a memory buffer then we'll do some additional setup and show a picture on the screen.

## Supplemental Reading

The usual guide for Vulkan learning is <https://vulkan-tutorial.com/>, and it's fine.

I've recently been pointed to <https://vkguide.dev/>, which also seems good.

They both cover the basics and both use "simple" C++ that you should be able to follow.

In terms of material from Khronos itself,
the general [SDK documentation](https://vulkan.lunarg.com/doc/sdk)
has links to all sorts of other material.
Of note is [The Vulkan Spec](https://renderdoc.org/vkspec_chunked/index.html).
It's pretty readable, and it is the official documentation for how all the functions work.
This link goes to a "chunked" version, where the info is split across a bunch of pages.
The SDK also has options for single-page HTML, and PDF.
