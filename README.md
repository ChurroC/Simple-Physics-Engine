BALLLZZZ
SIMPLE
LINK / CHAINS
SOFT BODY DYNAMICS - BODDIIES


TBH do I use self.last_dt or the current dt the change velocity
Fix up add_velocoty

try storing displacment instead of last

https://www.gafferongames.com/post/fix_your_timestep/ - Peak
https://leanrada.com/notes/sweep-and-prune/
https://developer.nvidia.com/gpugems/gpugems3/part-v-physics-simulation/chapter-32-broad-phase-collision-detection-cuda
https://lisyarus.github.io/blog/posts/perfect-collisions.html#section-you-spin


colliisn keep causing eahc object to move then can't apply gravity

use vectors from std lib


why use last dt for vel and accel

Thinkcing of using contirnous collison detection
but it would take a stupid amount of time in my use case
since we would need to cut each dt up by every collision time


TODO:
Chains

Make the color output be based of a gaussian output using the radius of the ball

implent CCD for high speeds

Sweep-and-prune (final)
idea from https://leanrada.com/notes/sweep-and-prune-2/


// 1. Broad phase - quick check using spatial grid
// 2. Medium phase - swept AABB check
// 3. Narrow phase - Only do expensive CCD on likely collisions
// 4. Batch handle collisions in time-sliced chunks

Fluid dynamics
Rigid body dynamics
Planet simulations