// Up until now, we have created command buffers that perform two kind of operations:

// - Memory transfers (copying data between buffers and images, clearing an image).
// - Compute operations (dispatching a compute shader).

// While these two kind of operations are sufficient in order to use the power of the GPU for parallel
// calculations (as seen in the Mandelbrot example), there is a third kind of operations: graphical operations.

// Before they were used for general-purpose calculations, GPUs were used for graphics (hence their name).
// To benefit from this, GPUs provide to developers a specialized well-optimized series of steps called
// the graphics pipeline. Using the graphics pipeline is more restrictive than using compute operations,
// but it is also much faster.

// > Note: There is nothing that the graphics pipeline can do that a compute pipeline couldn't do.
// > However the graphics pipeline is much more specialized and therefore much more optimized.
// > Some parts of the graphics pipeline are generally handled by dedicated chips on the hardware.
