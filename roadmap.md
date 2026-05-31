# ASCII 3D Renderer — Revised Roadmap (Ray Casting)

A from-scratch learning project: build a real-time software 3D renderer that outputs to a terminal using ASCII characters for shading. Walk through a 3D scene rendered entirely in text.

This roadmap replaces scanline rasterization with **ray casting** as the core rendering strategy. At terminal resolutions (~200×50 cells), casting one ray per cell is cheap, simple, and gives you per-cell geometric information that's hard to get from a rasterizer — which matters a lot for shape-aware character selection.

**What carries forward from the original roadmap:** your framebuffer abstraction and the ANSI cursor-reset approach. Bresenham's algorithm won't be in the render loop, but the knowledge isn't wasted — it's the same integer-arithmetic thinking you'll use elsewhere.

---

## Phase 1: Terminal Framebuffer ✅ (Done)

You already have this:

- 2D array of `char` representing terminal cells
- `clear()`, `set_pixel(x, y, char)`, `render()` (flush to stdout)
- ANSI `\033[H]` cursor reset (no flicker)
- Double buffering (build in memory, flush at once)

**One addition to make now:** store a `brightness: f32` (or equivalent) alongside each `char` in the buffer. You'll need it in Phase 3. A cell becomes `(char, brightness)` rather than just `char`.

---

## Phase 2: Casting Rays into a Scene

Get rays going from a camera through each terminal cell and intersecting geometry.

- **Define a camera:** position (eye point), forward/right/up basis vectors, and a field of view angle.
- **Generate one ray per cell:** for cell `(col, row)`, map it to normalized coordinates `(-1..1, -1..1)`, apply the aspect ratio correction (terminal cells are ~1:2), construct a direction vector from the camera through that point on the near plane.
- **Implement ray-sphere intersection.** A sphere is the simplest primitive — the math is a quadratic equation, ~10 lines of code. Return the hit distance `t` and the hit point.
- **Compute the surface normal** at the hit point (for a sphere: `normalize(hit_point - center)`).
- **Render it:** if a ray hits, write `#` to the framebuffer; if it misses, write a space.

You should now see a circle on screen. It won't look 3D yet — that comes from lighting.

**Research areas:**
- Ray-sphere intersection (deriving the quadratic from the parametric ray equation and the sphere equation)
- Camera ray generation (mapping pixel/cell coordinates to world-space ray directions)
- Normalized device coordinates and aspect ratio correction
- Vector math essentials: dot product, normalization, cross product

---

## Phase 3: Lighting and ASCII Shading

Make the sphere look three-dimensional.

- **Directional light:** define a light as a normalized direction vector.
- **Diffuse shading:** `brightness = max(0.0, dot(normal, light_dir))`. This is Lambertian reflectance — brightness is proportional to the cosine of the angle between the surface and the light.
- **Add ambient light:** a small constant (e.g. `0.1`) added to the diffuse term so surfaces facing away from the light aren't completely invisible.
- **Map brightness to a character ramp:**
  ```
  " .:-=+*#%@"
  ```
  Index into this string by `brightness * (ramp_length - 1)`. Low density = dim, high density = bright.
- **Store the brightness** in your framebuffer cell for later use.

You should now see a shaded sphere — clearly 3D, with a bright side and a dark side. This is the core visual payoff of ray casting: per-cell normals and lighting come naturally.

**Research areas:**
- Lambertian / diffuse reflectance model
- Character density ramps (choosing and ordering characters by visual "weight")
- Ambient vs. diffuse vs. specular lighting (Phong/Blinn-Phong for specular — stretch goal)
- Gamma / perceptual brightness (the eye doesn't perceive linear brightness linearly — you may want a nonlinear ramp mapping)

---

## Phase 4: A Scene with Multiple Primitives

Go from one sphere to a world you can populate.

- **Ray-plane intersection** — an infinite ground plane gives spatial grounding. The math is even simpler than the sphere case.
- **Ray-axis-aligned box (AABB) intersection** — boxes are useful as walls, obstacles, and bounding volumes. The slab method tests against three axis-aligned slabs and is fast.
- **Scene as a list of primitives:** loop through all objects, test each ray against all of them, keep the closest hit. This is your "scene traversal" — naive but fine for a small number of objects.
- **Material per object:** assign each primitive a character ramp, color, or reflectance so they're visually distinguishable.
- Build a simple test scene: a ground plane, a few spheres, a couple of boxes. Confirm depth ordering works (closer objects occlude farther ones naturally — the closest-hit query handles this).

**Research areas:**
- Ray-plane intersection (parametric ray vs. plane equation)
- Ray-AABB intersection (slab method / Kay-Kajiya)
- Closest-hit vs. any-hit queries (the distinction matters for shadows later)
- Scene representation (flat list now; spatial acceleration later)

---

## Phase 5: Camera Movement and Input

Make the scene walkable.

- **Terminal raw mode:** read keypresses without waiting for Enter. Use `crossterm` (Rust), `ncurses` (C), or equivalent.
- **First-person controls:** WASD for movement along the camera's forward/right vectors. Mouse or arrow keys for look direction (yaw and pitch).
- **View matrix:** construct from the camera's position and orientation. You don't strictly need a matrix here — you can rebuild the basis vectors (forward, right, up) from yaw/pitch angles each frame — but understanding the view matrix is valuable.
- **Frame timing:** measure frame duration, multiply movement by delta time so speed is framerate-independent. Add a sleep to cap at ~30fps and avoid burning CPU.
- **Query terminal size** dynamically (`ioctl` / `TIOCGWINSZ` / library call) and resize the framebuffer if the terminal changes.

At this point you have a real-time walkable 3D environment. Everything after this is about making it look better and handle more complex scenes.

**Research areas:**
- Terminal raw mode and keypress reading (platform differences)
- Euler angles (yaw/pitch) for FPS-style camera control
- Delta time and fixed vs. variable timestep
- The MVP pipeline conceptually (Model → View → Projection), even though in a ray caster you apply the inverse: you transform rays rather than geometry

---

## Phase 6: Triangles and Meshes

Move beyond analytic primitives to arbitrary geometry.

- **Ray-triangle intersection:** implement Möller–Trumbore. It's the standard algorithm — fast, no pre-computation, gives you barycentric coordinates for free.
- **Barycentric coordinates** let you interpolate any per-vertex attribute (normals, UVs, colors) across the triangle surface. Store the barycentrics from the hit test.
- **Load OBJ files:** the format is simple — `v` lines for vertices, `vn` for normals, `f` for faces. Parse them into a list of triangles.
- **Per-vertex normals + smooth shading:** if the OBJ has vertex normals, interpolate them across the triangle using barycentrics. This gives you smooth Gouraud-style shading on curved surfaces instead of faceted flat shading.
- **This is where you need acceleration.** A mesh with 1000 triangles × 10,000 rays = 10M intersection tests per frame. You need a BVH.

**Research areas:**
- Möller–Trumbore ray-triangle intersection
- Barycentric coordinates (what they are, how to interpolate with them)
- OBJ file format (vertices, normals, faces, and optionally UVs)
- Bounding Volume Hierarchy (BVH) — the standard spatial acceleration structure for ray tracing. Build a binary tree of AABBs; each ray traverses the tree and only tests triangles in boxes it hits. This is the single biggest performance lever in the entire project.
- SAH (Surface Area Heuristic) for BVH construction quality

---

## Phase 7: Shape-Aware Character Selection

This is the payoff for choosing ray casting: you can super-sample each cell and use the sub-cell geometry to pick characters that represent *shape*, not just brightness.

**The idea:** a single ray per cell tells you brightness. Multiple rays per cell tell you *where within the cell* geometry exists. A cell where geometry fills the left half should render as `▌`; a diagonal edge should render as `/` or `\`; a corner should render as `┐`. This simulates resolution higher than the character grid.

**Approach — sub-cell sampling:**

1. Cast a grid of sample rays within each cell (e.g., 4×8, matching the approximate pixel aspect ratio of a terminal character).
2. Record which sub-cell samples hit geometry and which don't (binary: hit or miss against the primary surface). This gives you a 1-bit "coverage mask" per cell.
3. Match the coverage mask against a library of known character bitmasks. Pick the character whose bitmask is closest (by Hamming distance or similar metric).

**Character library candidates:**
- **Box-drawing characters** (`─ │ ┌ ┐ └ ┘ ├ ┤ ┬ ┴ ┼`) — great for hard edges.
- **Block elements** (`▀ ▄ ▌ ▐ █ ░ ▒ ▓`) — good for area coverage.
- **Braille patterns** (`⠁⠂⠃...⣿`) — 256 patterns mapping to a 2×4 dot grid. Effectively 8× the resolution of plain ASCII. These are the highest-fidelity option if your terminal supports Unicode.
- **Diagonal/slope characters** (`/ \ | — _ .`) — for edge direction.

**Combining shape and brightness:**
You now have two signals per cell — *shape* (coverage mask) and *brightness* (lighting). Options:
- Use shape for character selection and ANSI color/grayscale for brightness.
- Use shape for character selection from a brightness-sorted subset (e.g., among characters that match the coverage pattern, pick the one closest in visual density to the target brightness).
- If restricted to pure ASCII without color, prioritize shape at edges (where it matters most for perceived resolution) and fall back to luminance-only ramp on interior surfaces.

**Research areas:**
- Sub-cell / super-sampling strategies (regular grid, jittered, stratified)
- Coverage masks and bitmap matching (Hamming distance, pre-computed lookup tables)
- Unicode block elements and Braille character encoding (the Braille codepoint is literally a bitmask: `U+2800 + dot_bits`)
- Anti-aliasing concepts (what you're doing is essentially hand-crafted AA at the character level)
- Edge detection from coverage masks (identify edge direction from the boundary between hit/miss samples)

---

## Phase 8: Shadows, Reflections, and Visual Polish

Stretch rendering further now that the core pipeline is solid.

- **Hard shadows:** from each hit point, cast a secondary ray toward the light. If it hits anything, the point is in shadow — reduce brightness to ambient only. This is an "any-hit" query (you can early-out on the first intersection), so it's cheaper than the primary closest-hit.
- **Reflections:** at a hit point, compute the reflection direction (`r = d - 2*dot(d,n)*n`), cast a new ray, blend the result. Limit recursion depth (1–2 bounces is plenty).
- **Fog / depth cue:** fade distant surfaces toward a background character or reduce brightness with distance. Gives strong depth perception in a terminal.
- **Floor grid pattern:** alternate the ground plane's character or brightness in a checkerboard based on world-space coordinates. Classic ray casting visual — costs nothing and adds spatial reference.
- **Multiple lights:** sum contributions from multiple light sources. Colored lights with ANSI color output.

**Research areas:**
- Shadow rays and shadow acne (offset the ray origin slightly along the normal to avoid self-intersection)
- Reflection vector computation
- Recursive ray tracing (and when to stop recursing)
- Atmospheric attenuation / fog models

---

## Phase 9: Performance

Keep the frame rate real-time as scene complexity grows.

- **BVH optimization** (if not done in Phase 6): this is your biggest win. A well-built BVH turns O(n) triangle tests into O(log n) per ray.
- **Parallelism:** each ray is independent — this is embarrassingly parallel. Split rows (or blocks of cells) across threads. Rayon (Rust) or a thread pool makes this straightforward.
- **Early termination:** if a cell is fully occluded or in deep shadow, skip expensive shading.
- **Adaptive sampling for Phase 7:** only run the sub-cell super-sampling near edges (where coverage masks actually vary). Interior cells that are fully covered can use the fast single-ray luminance path. Detect edges by checking whether neighboring cells hit different objects or have large normal discontinuities.
- **Spatial coherence:** neighboring cells tend to hit the same object. Cache the last-hit primitive and test it first for the next ray (a "mailboxing" / "ray coherence" optimization).
- **Profile before optimizing.** Measure where time actually goes — it's almost always intersection testing, and almost always fixed by BVH quality.

**Research areas:**
- BVH traversal optimization (stack-based vs. stackless, packet traversal)
- Thread-level parallelism for ray casting (work partitioning strategies)
- Adaptive super-sampling (edge detection to decide where to spend samples)
- SIMD for vector math (worth it if you're in C/C++/Rust; less impactful in higher-level languages)
- Profiling tools (`perf`, `flamegraph`, `cargo-flamegraph`)

---

## Stretch Goals

Natural extensions once the core pipeline is working:

- **Constructive Solid Geometry (CSG)** — combine primitives with boolean operations (union, intersection, subtraction) for complex shapes without meshes
- **Signed Distance Fields (SDFs)** — represent geometry as distance functions, ray march instead of ray cast. Enables smooth blending, infinite repetition, and procedural shapes with no mesh data at all
- **Texture mapping** — interpolate UV coordinates via barycentrics, sample a texture (even an ASCII-art texture)
- **Skybox / environment mapping** — for rays that miss all geometry, sample a background based on direction
- **Scene file format** — define scenes in a config file (TOML/JSON) instead of hardcoding geometry
- **ANSI truecolor** — combine character shape with 24-bit color for dramatically richer output
- **Portals / non-Euclidean geometry** — when a ray hits a portal surface, teleport and redirect it. Terminal rendering makes this surreal effect especially striking

---

## Architectural Summary

The pipeline is clean and linear:

```
For each frame:
  For each cell (col, row):
    1. Generate ray      (camera + cell coords → ray origin & direction)
    2. Trace ray          (intersect scene → closest hit, normal, material)
    3. Shade              (lighting → brightness float)
    4. [Optional] Super-sample sub-cell  (coverage mask → shape info)
    5. Select character   (brightness + shape → char)
    6. Write to framebuffer
  Flush framebuffer to terminal
  Read input, update camera
```

Every cell is independent. The ray carries all the information you need — hit point, normal, depth, material — and you never need to project, clip, or rasterize a triangle. The 2D rasterization code from Phase 2 of the old roadmap isn't wasted knowledge, but it's no longer in the hot path.

The old roadmap's key insight ("the 2D rasterizer is the same code that draws 3D faces") is replaced by a simpler one: **the ray is the only abstraction.** Primary rays do visibility. Shadow rays do lighting. Reflection rays do materials. Sub-cell sample rays do character selection. It's rays all the way down.
