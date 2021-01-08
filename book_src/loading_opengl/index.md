
# Loading Open GL

While the *usage* of Open GL is basically the same across platforms,
the precise details of how to first initialize Open GL varies.

The general idea is that we need to do two things:

1) Create an [Open GL](https://www.khronos.org/opengl/wiki/OpenGL_Context) context and make it "current".
  A GL context is what holds all of the drawing state for GL.
  Each GL Context can only be current in a single thread at a time.
  Also, the thread's current context is a thread-local variable,
  so you can't have more than one context current in a thread at the same time.
  Also, a context is associated with a particular window's drawing area.
  Usually you have just one window,
  and so you have only one GL context,
  meaning that don't need to worry about any of that.
  If you're trying to use GL with more than one window at a time,
  things can get tricky.
2) Load the GL function pointers.
  Unfortunately, you can't access GL like it was a normal dynamic library.
  At least, you can't on Windows and Linux.
  We'll have a whole fun time [loading function pointers](https://www.khronos.org/opengl/wiki/Load_OpenGL_Functions) manually.
