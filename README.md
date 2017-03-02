hello_evergreen
===============

This was my attempt at finding out how the Linux graphics stack works. This code can draw a green square using ioctls to the GPU only. It borrows from [radeondemo](https://cgit.freedesktop.org/~airlied/radeondemo/) and [Mesa](https://www.mesa3d.org/).
This will probably only work with AMD Evergreen GPUs.

TODO:

* Allow specification of resolution
* Tiled framebuffers
* Vsyncing
