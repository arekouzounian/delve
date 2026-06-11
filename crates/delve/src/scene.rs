use std::collections::HashMap;
use std::time::Instant;

use crate::entities::{Collision, Entity, Shape};
use delve_shared::math::{Mat3, Vec3};

pub struct Camera {
    position: Vec3,
    target: Vec3,
    pub velocity: Vec3,
    fov_radians: f32,
}

impl Camera {
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
    start: Instant,
}

impl Scene {
    /// fov_rad: field of view angle, in radians
    pub fn new(camera: Camera, ambient_lighting: f32) -> Self {
        Self {
            camera,
            objects: HashMap::new(),
            lights: Vec::new(),
            ambient_lighting,
            start: Instant::now(),
        }
    }

    // TODO: object de-registration/ID tracking
    pub fn register_shape(&mut self, key: String, shape: Shape) {
        self.objects.insert(key, shape);
    }

    pub fn register_light(&mut self, light: Light) {
        self.lights.push(light);
    }

    pub fn lambertian_brightness(&self, hit: &Collision) -> f32 {
        let diffuse = self
            .lights
            .iter()
            .map(|light| hit.surface_normal.dot(light.direction).max(0.0) * light.intensity)
            .sum::<f32>();

        // camera is also a light
        // lets assume half angle is 30 degrees = pi/5
        // refactor this later its gonna be inefficient
        let half_angle = (std::f32::consts::PI / 5.0).cos();
        let ray = hit.collision_position - self.camera.position;
        let in_cone = ray.dot(self.camera.forward_normal()) <= half_angle;
        let attenuation = (1.0 / (ray.square_magnitude())).min(1.0);
        let camera_brightness = 0.5;

        let mut result = self.ambient_lighting + diffuse;

        if in_cone {
            result += attenuation * camera_brightness;
        }

        result.min(1.0)
    }

    pub fn update_all(&mut self) {
        for shape in self.objects.values_mut() {
            match shape {
                Shape::Cube(c) => c.sin_hover(self.start),
                Shape::RectPrism(r) => r.set_rotation(Mat3::from_axis_angle(
                    Vec3::Z,
                    self.start.elapsed().as_secs_f32(),
                )),
                _ => (),
            }
        }
    }

    // TODO: naively checks all shapes; should optimize this
    // maybe use rayon?
    pub fn intersect(&self, ray_origin: Vec3, ray_direction: Vec3) -> Option<Collision> {
        let mut closest: Option<Collision> = None;

        for shape in self.objects.values() {
            let result = match shape {
                Shape::Sphere(s) => s.intersect(ray_origin, ray_direction),
                Shape::RectPrism(r) => r.intersect(ray_origin, ray_direction),
                Shape::Cube(c) => c.intersect(ray_origin, ray_direction),
            };

            if let Some(collision) = result {
                match closest.take() {
                    None => closest = Some(collision),
                    Some(c) => closest = Some(Collision::closest(collision, c)),
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
