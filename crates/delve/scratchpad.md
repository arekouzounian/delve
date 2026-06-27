# scratchpad
---

Implementing physics.
So the first thing that comes to mind is that now, instead of position, we also have velocity and acceleration vectors.
Maybe I can have a trait that's like 'Physics' that applies acceleration/velocity each tick. the main consideration is decay, like
there'll be some acceleration that naturally decays. there'll also be objects that we want to move that don't have acceleration, and
we want to just set their position/rotation manually

The other hard thing to consider is collision. there's no event loop/event-driven system; how to settle collisions?
collisions shouldn't be too bad. i guess while we're calculating rays we can also calculate collisions between each object; naively,
using polynomial time we iterate through each object and see if it collides with each other object. Without some sort of space partitioning
algorithm i actually don't think we can do better than that. we also have to take into account the object's rotation

I guess if we're paring down the physics system to just the player then we don't have to worry about the collisions of other objects; they can
just clip into each other. Gravity should be simple to calculate/abide by for all objects (as long as we define a ground), but we can just skip
the implementation for other object collisions and just focus on the player model.

honestly if we use rayon carefully in our render loop then we might not need gpu accel.
really the only computationally expensive thing is calculating rays and drawing them to the screen.
if we pare down collisions to just collisions on the camera then we can get really solid performance

the interface should be such that we can register objects at runtime (because we're building towards procedural generation). That's pretty much
what it already is but i just wanna keep that in mind. so here's what we have:
- A __current__ room; this can of course change
- Each room is filled with objects, each of which has a position, a rotation, and optionally velocity and acceleration.
- We also have a camera, which is just like any other object except for the following characteristics:
  - all raycasts come from the camera's perspective
  - input directly manages the object's velocity
  - the camera actually calculates collisions against other objects, and cannot occupy the same space as other objects
- there should be some central ground, and the player should never be below the ground.
  - if the player ever goes below, they should be teleported back up above the ground
- objects should also have a way to opt in & out of dynamic lighting. that is, later on when i simulate textures atop a surface, I should
be able to just keep the brightness as a constant for that surface/object/etc.
- lighting should be able to have variable reflectiveness; i.e. ground should not always be fully bright
  - i think that means storing some absorption on each collision object
