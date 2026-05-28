# ASCII 3D Renderer — Project Roadmap

A from-scratch learning project: build a software 3D renderer that outputs to a terminal using ASCII characters for shading. Each phase produces something visible and builds directly on the last.

---

## Phase 1: Terminal Framebuffer

Get characters on screen in a controlled way.

- Build a framebuffer abstraction: a 2D array of `char` (or equivalent) representing terminal cells
- Implement `clear()`, `set_pixel(x, y, char)`, and `render()` (flush the whole buffer to stdout)
- Use ANSI escape `\033[H` to reset the cursor to the top-left each frame instead of clearing the terminal (avoids flicker)
- Test by drawing hardcoded patterns — confirm you can place arbitrary characters at arbitrary positions

**Research areas:**
- ANSI escape codes (cursor movement, color, alternate screen buffer)
- Double buffering (build the frame in memory, flush all at once)
- Terminal cell aspect ratio (~1:2 width:height — this matters later)
- Raw mode vs cooked mode (relevant once you add input)

---

## Phase 2: 2D Rasterization

Draw lines and filled shapes on your framebuffer.

- Implement **Bresenham's line algorithm** — draws a line between two integer points using only addition and comparison, no floating point
- Draw wireframe shapes by connecting vertices with lines
- Implement **scanline triangle fill**: for each row of the triangle, compute the left and right edges, fill the span between them
- Draw a rectangle as two triangles to confirm your fill works

**Research areas:**
- Bresenham's line algorithm (integer-only rasterization, octant handling)
- Scanline rasterization (edge tables, span filling, flat-top/flat-bottom triangle decomposition)
- Winding order (clockwise vs counterclockwise vertex ordering — determines front/back later)
- Subpixel precision (not critical at terminal resolution, but good to know about)

---

## Phase 3: 2D Transformations

Move, rotate, and scale shapes using matrix math.

- Represent 2D transforms as 3×3 homogeneous matrices (the extra dimension lets you encode translation as a multiply)
- Implement rotation, translation, and scaling matrices
- Multiply your shape's vertices by the matrix before rasterizing
- Animate: increment the rotation angle each frame to see a spinning shape

**Research areas:**
- Homogeneous coordinates (why a 2D transform uses a 3×3 matrix)
- Affine transformations (rotation, translation, scale, shear)
- Matrix multiplication (row-major vs column-major, composition order)
- Radians vs degrees, `sin`/`cos` for rotation matrices

---

## Phase 4: 3D Wireframe with Perspective

Define 3D geometry and project it onto 2D.

- Define a cube as 8 vertices in (x, y, z) and a list of edges (pairs of vertex indices)
- Apply a **4×4 rotation matrix** to spin the cube in 3D
- Apply **perspective projection**: divide x and y by z to get screen coordinates (objects farther away appear smaller)
- Draw the projected edges as lines using your Phase 2 line algorithm
- Apply the aspect ratio correction (scale y by ~0.5) so the cube doesn't look squashed

**Research areas:**
- 4×4 transformation matrices (model matrix: translate, rotate, scale in 3D)
- Perspective projection (the "divide by z" operation, field of view, near/far planes)
- Clip space and Normalized Device Coordinates (NDC)
- Perspective divide (dividing by the w component after matrix multiply)
- Euler angles vs rotation matrices (gimbal lock is worth knowing about, even if you stick with matrices for now)

---

## Phase 5: Filled Faces + Depth Buffer

Transition from wireframe to solid surfaces.

- Define the cube's 6 faces as 12 triangles (2 per face), using vertex indices
- Rasterize each triangle as a filled shape using your scanline algorithm from Phase 2
- Implement a **depth buffer** (Z-buffer): a parallel 2D array of floats, same dimensions as the framebuffer, initialized to infinity each frame
- When filling a pixel, **interpolate the depth** across the triangle and only write if the new depth is closer than the stored value
- Implement **back-face culling**: compute the face's normal via the cross product of two edges; dot it with the view direction; skip the face if it points away

**Research areas:**
- Z-buffer algorithm (per-pixel depth testing, depth interpolation across a triangle)
- Barycentric coordinates (an alternative to scanline — lets you interpolate any value across a triangle, including depth)
- Cross product (computes a vector perpendicular to a triangle's surface)
- Dot product (measures alignment between two vectors — used for culling and lighting)
- Surface normals (per-face vs per-vertex normals, flat shading vs smooth shading)
- Painter's algorithm (an older alternative to depth buffering — sort faces back-to-front and draw in order; simpler but breaks with overlapping geometry)

---

## Phase 6: Lighting and ASCII Shading

Make the flat faces look three-dimensional using light.

- Define a directional light as a normalized 3D vector
- For each face, compute **diffuse shading**: `brightness = max(0, dot(face_normal, light_direction))`
- Map the brightness float (0.0–1.0) to an index in a character ramp, e.g.:
  ```
  . , - ~ ; = ! * # $ @
  ```
  (low density = dim, high density = bright)
- Each face now renders with a different character, giving the illusion of depth and lighting

**Research areas:**
- Lambertian / diffuse reflectance model (brightness proportional to cosine of angle between normal and light)
- Character density ramps (choosing and ordering characters by visual weight)
- Ambient light (a constant minimum brightness so shadowed faces aren't invisible)
- Specular highlights (Phong or Blinn-Phong reflection model — a stretch goal)
- Smooth shading / Gouraud shading (interpolate brightness per-vertex instead of per-face for curved surfaces — requires per-vertex normals)

---

## Phase 7: Camera, Input, and Polish

Make it interactive and robust.

- Implement a **view matrix** (inverse of the camera's transform — moves the world so the camera is at the origin)
- Add keyboard input: arrow keys to rotate the camera or object, WASD to move
- Query terminal dimensions dynamically (`ioctl` / ANSI `\033[18t` / library call) and resize framebuffers on change
- Add a frame rate limiter (sleep to hit a target like 30fps)
- Consider using a library like `crossterm` (Rust) or `ncurses` (C) for robust input handling and terminal control

**Research areas:**
- View matrix (look-at matrix construction from eye position, target, and up vector)
- The full MVP pipeline: Model matrix → View matrix → Projection matrix (and the order of multiplication)
- Terminal raw mode (reading keypresses without waiting for Enter)
- `ioctl` / `TIOCGWINSZ` (querying terminal size on Unix)
- Frame timing (fixed timestep vs variable timestep, delta time)

---

## Stretch Goals

Once the core pipeline works, there are natural extensions:

- **OBJ file loading** — parse `.obj` files to render arbitrary meshes (the format is simple: lists of `v` vertices and `f` faces)
- **Texture mapping** — interpolate UV coordinates across triangles, sample from a 2D texture (even ASCII art textures)
- **Multiple objects** — each with its own model matrix
- **Clipping** — properly clip triangles that are partially behind the camera (currently they'll produce garbage projections)
- **Frustum culling** — skip objects entirely if they're outside the camera's view
- **Color** — terminals support ANSI 256-color and truecolor; combine color with character density for much richer output

---

## Key Architectural Insight

The 2D rasterizer you build in Phase 2 is the **same code** that draws 3D faces in Phase 5. The pipeline has two clean halves:

1. **3D → 2D math** (phases 4–7): transforms, projection, lighting — produces 2D triangles with depth and brightness
2. **2D → screen** (phases 1–2): rasterization, framebuffer — takes 2D triangles and fills characters

You're not replacing the 2D renderer when you "go 3D." You're building a front end that feeds into it.
