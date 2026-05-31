use std::collections::HashMap;

use crate::util::min;
use crate::vector::Vec3;

pub struct Camera {
    position: Vec3,
    target: Vec3,
    pub velocity: Vec3,
    fov_radians: f32,
}

impl Camera {
    // all from memory. wow
    pub const PI: f32 = 3.141592653;

    pub const WORLD_UP: Vec3 = Vec3::new(0.0, 1.0, 0.0);

    pub fn new(position: Vec3, target: Vec3, fov_radians: f32) -> Self {
        Self {
            position,
            target,
            velocity: Vec3::ORIGIN,
            fov_radians,
        }
    }

    pub fn get_position(&self) -> Vec3 {
        self.position
    }

    pub fn fov_factor(&self) -> f32 {
        f32::tan(self.fov_radians / 2.0)
    }

    pub fn forward_normal(&self) -> Vec3 {
        (self.target - self.position).normalize()
    }

    pub fn right_normal(&self) -> Vec3 {
        Vec3::cross_product(&Self::WORLD_UP, &self.forward_normal()).normalize()
    }

    pub fn up_normal(&self) -> Vec3 {
        Vec3::cross_product(&self.forward_normal(), &self.right_normal())
    }

    pub fn apply_force(&mut self, force_vec: Vec3) {
        self.position += force_vec;
        self.target += force_vec;
    }

    pub fn apply_yaw(&mut self, angle_rads: f32) {
        let forward = self.target - self.position;
        let yawed = Vec3::new(
            forward.x * angle_rads.cos() - forward.z * angle_rads.sin(),
            forward.y,
            forward.x * angle_rads.sin() + forward.z * angle_rads.cos(),
        );

        self.target = self.position + yawed;
    }

    // positive angle goes up, negative goes down
    pub fn apply_pitch(&mut self, angle_rads: f32) {
        let forward = self.target - self.position;
        let right = self.right_normal();
        // rodrigues' rotation formula, i guess
        // rodrigues was onto something
        let pitched = forward.scalar_multiply(angle_rads.cos())
            + Vec3::cross_product(&right, &forward).scalar_multiply(angle_rads.sin())
            + right.scalar_multiply(right.dot(forward) * (1.0 - angle_rads.cos()));

        let new_forward = pitched.normalize();
        let vertical = new_forward.dot(Self::WORLD_UP);
        if vertical.abs() > 0.99 {
            return; // clamp
        }

        self.target = self.position + pitched;
    }
}

pub struct Sphere {
    position: Vec3,
    radius: f32,
}

#[allow(unused)]
/// do we need any of this other stuff? we may want to just return the surface normal
pub struct Collision {
    collision_position: Vec3,
    surface_normal: Vec3,

    // origin point of the ray + length_coefficient * direction vector
    // this gives us the position of the hit point
    // do we need this?
    length_coefficient: f32,
}

impl Collision {
    pub fn get_surface_normal(&self) -> Vec3 {
        self.surface_normal
    }
}

impl Sphere {
    pub fn new(position: Vec3, radius: f32) -> Self {
        Self { position, radius }
    }

    /// The way this works is by solving for the points that the ray would
    /// collide with the sphere. In this case, we only return a collision
    /// if there are exactly two points that the ray collides with the sphere
    /// (entry and exit point), and those points are both positive (in front of the camera).
    /// the collision is at the closest point, the entry.
    /// ray_direction should be normalized
    pub fn intersect(&self, ray_origin: Vec3, ray_direction_normal: Vec3) -> Option<Collision> {
        let origin_center_diff = ray_origin - self.position;

        // formula: at^2 + bt + c
        // technically a is not needed because it's normalized, and always 1
        let a = ray_direction_normal.dot(ray_direction_normal);
        let b = 2.0 * origin_center_diff.dot(ray_direction_normal);
        let c = origin_center_diff.dot(origin_center_diff) - (self.radius * self.radius);

        let discriminant = (b * b) - (4.0 * a * c);

        if discriminant <= 0.0 {
            return None;
        }

        let length_coefficient_one = (-b + discriminant.sqrt()) / (2.0 * a);
        let length_coefficient_two = (-b - discriminant.sqrt()) / (2.0 * a);

        if length_coefficient_one < 0.0 || length_coefficient_two < 0.0 {
            return None;
        }

        let length_coefficient = min(length_coefficient_one, length_coefficient_two);
        let collision_position =
            ray_origin + ray_direction_normal.scalar_multiply(length_coefficient);
        let surface_normal = (collision_position - self.position).normalize();

        Some(Collision {
            collision_position,
            surface_normal,
            length_coefficient,
        })
    }
}

pub enum Shape {
    Sphere(Sphere),
}

pub struct Light {
    direction: Vec3, // point towards light, not from
    intensity: f32,  // 0.0-1.0
}

impl Light {
    pub fn new(direction: Vec3, intensity: f32) -> Self {
        Self {
            direction: direction.normalize(),
            intensity,
        }
    }
}

pub struct Scene {
    camera: Camera,
    objects: HashMap<String, Shape>,
    lights: Vec<Light>,
    ambient_lighting: f32,
}

impl Scene {
    /// fov_rad: field of view angle, in radians
    pub fn new(camera: Camera, ambient_lighting: f32) -> Self {
        Self {
            camera,
            objects: HashMap::new(),
            lights: Vec::new(),
            ambient_lighting,
        }
    }

    // TODO: object de-registration/ID tracking
    pub fn register_shape(&mut self, key: String, shape: Shape) {
        self.objects.insert(key, shape);
    }

    pub fn register_light(&mut self, light: Light) {
        self.lights.push(light);
    }

    pub fn lambertian_brightness(&self, surface_normal: Vec3) -> f32 {
        let diffuse = self
            .lights
            .iter()
            .map(|light| surface_normal.dot(light.direction).max(0.0) * light.intensity)
            .sum::<f32>();

        (self.ambient_lighting + diffuse).min(1.0)
    }

    // TODO: naively checks all shapes; should optimize this
    // maybe use rayon?
    pub fn intersect(&self, ray_origin: Vec3, ray_direction: Vec3) -> Option<Collision> {
        let mut closest: Option<Collision> = None;

        for (_id, shape) in &self.objects {
            let result = match shape {
                Shape::Sphere(s) => s.intersect(ray_origin, ray_direction),
            };

            if let Some(collision) = result {
                match closest.take() {
                    None => closest = Some(collision),
                    Some(c) => {
                        if c.length_coefficient > collision.length_coefficient {
                            closest = Some(collision)
                        }
                    }
                };
            }
        }

        closest
    }

    /// Returns forward normal, right normal, up normal
    pub fn get_normals(&self) -> (Vec3, Vec3, Vec3) {
        (
            self.camera.forward_normal(),
            self.camera.right_normal(),
            self.camera.up_normal(),
        )
    }

    pub fn get_camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    pub fn get_camera(&self) -> &Camera {
        &self.camera
    }
}
