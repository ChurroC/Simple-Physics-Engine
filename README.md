BALLLZZZ
SIMPLE
LINK / CHAINS
SOFT BODY DYNAMICS - BODDIIES
3D BALLZ
Audio Visualizer using planetary forces as msuics volume
Use this to model a ball balance, pendulum, spider walker machine language


https://www.gafferongames.com/post/fix_your_timestep/ - Peak
https://leanrada.com/notes/sweep-and-prune/
https://developer.nvidia.com/gpugems/gpugems3/part-v-physics-simulation/chapter-32-broad-phase-collision-detection-cuda
https://lisyarus.github.io/blog/posts/perfect-collisions.html#section-you-spin
https://gameprogrammingpatterns.com/spatial-partition.html
https://gameidea.org/2024/02/02/physics-collision-detection/#Space_partitioning

IDEAS:
use threads to run the colliison detection after the update. so it checks while the frames are being generated. so by the next tien the dt finished we will already have collisons detected.

TODO:
parallize the space partioning
chains

FOLDERS:
main-engine - This is the one I started of with and has all my ideas and all the different codes
simple-engine - This is my best simple engine with space partioning
constraint-engine - This is the same as simple-engine but with contraints for links and soft bodies
3D-engine - This is the same as constraint-engine but in 3D with WGPU

FUTURE OTHER PROJECTS:
Fluid dynamics
Rigid body dynamics
Planet simulations
