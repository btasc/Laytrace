# Laytrace
Core wgpu-rust setup for my personal ray tracing related projects. Intended to be a rust library to simplify \
the wgpu interface for ray tracing as to reduce the amount of wgpu boilerplate and wgsl complexity. 

## Math conventions
The "simulation space" is set in a 3d coordinate system. The system is set up as to where the x-axis is \
right / left, the y-axis is up and down, and the z axis is forward and backward. Basically, imagine looking \
at a 2d plane, except that moving towards you is positive Z. 

The camera is stored with an x,y,z coordinate, and a pitch and yaw to handle directions. Pitch is simply \
the unit circle, where a pitch of 0 is on the positive x-axis, a pitch if pi/2 is positive y-axis, and so on. \
The yaw is similar. A yaw of 0 is pointing down positive x, a yaw of pi/2 is negative z, and so on.

