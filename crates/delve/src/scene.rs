use std::collections::HashMap;

use delve_macros::entity;
use delve_shared::{
    constants::INPUT_SCALE,
    math::Vec3,
    traits::{Entity, IntersectableEntity},
    types::Collision,
};

use crate::input::CameraForce;

#[entity]
pub struct Camera {
    target: Vec3,
    fov_radians: f32,
    height: f32,
    /// radius of the invisible cylinder hitbox
    radius: f32,
}

pub struct CameraNormals {
    pub forward: Vec3,
    pub right: Vec3,
    pub up: Vec3,
}

impl Camera {
    pub const WORLD_UP: Vec3 = Vec3::new(0.0, 1.0, 0.0);

    pub fn new(target: Vec3, fov_radians: f32, height: f32, radius: f32) -> Self {
        Self {
            target,
            fov_radians,
            height,
            radius,
            ..Self::default()
        }
    }

    pub fn fov_factor(&self) -> f32 {
        f32::tan(self.fov_radians / 2.0)
    }

    pub fn forward_normal(&self) -> Vec3 {
        (self.target - self.entity_fields.position).normalize()
    }

    pub fn right_normal(&self) -> Vec3 {
        Vec3::cross_product(&Self::WORLD_UP, &self.forward_normal()).normalize()
    }

    pub fn up_normal(&self) -> Vec3 {
        Vec3::cross_product(&self.forward_normal(), &self.right_normal())
    }

    pub fn update_target_and_forces(&mut self) {
        self.apply_forces();
        self.target += self.entity_fields.velocity;
    }

    pub fn apply_yaw(&mut self, angle_rads: f32) {
        let forward = self.target - self.entity_fields.position;
        let yawed = Vec3::new(
            forward.x * angle_rads.cos() - forward.z * angle_rads.sin(),
            forward.y,
            forward.x * angle_rads.sin() + forward.z * angle_rads.cos(),
        );

        self.target = self.entity_fields.position + yawed;
    }

    // positive angle goes up, negative goes down
    pub fn apply_pitch(&mut self, angle_rads: f32) {
        let forward = self.target - self.entity_fields.position;
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

        self.target = self.entity_fields.position + pitched;
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
    objects: HashMap<String, Box<dyn IntersectableEntity>>,
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
    pub fn register_shape(&mut self, key: String, entity: Box<dyn IntersectableEntity>) {
        self.objects.insert(key, entity);
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

        let result = self.ambient_lighting + diffuse;

        result.min(1.0)
    }

    pub fn update_all(&mut self) {
        // TODO: maybe we can have an api where you set an update closure on the entity
        // and this just calls it

        // for shape in self.objects.values_mut() {
        //     match shape {
        //         Shape::Cube(c) => c.sin_hover(self.start),
        //         Shape::RectPrism(r) => r.set_rotation(Mat3::from_axis_angle(
        //             Vec3::Z,
        //             self.start.elapsed().as_secs_f32(),
        //         )),
        //         _ => (),
        //     }
        // }
    }

    // TODO: naively checks all shapes; should optimize this
    // maybe use rayon?
    pub fn intersect(&self, ray_origin: Vec3, ray_direction: Vec3) -> Option<Collision> {
        let mut closest: Option<Collision> = None;

        for shape in self.objects.values() {
            if let Some(collision) = shape.intersect(ray_origin, ray_direction) {
                match closest.take() {
                    None => closest = Some(collision),
                    Some(c) => closest = Some(Collision::closest(collision, c)),
                };
            }
        }

        closest
    }

    /// calculate collisions as follows:
    /// loop through each object in the scene, find its closest point.
    /// if the closest point is inside us, don't worry, move freely
    ///   (need to be careful with this or else we can fall through the ground)
    ///
    /// If it's some external point, check if its colliding with our invisible
    /// cylinder
    pub fn apply_camera_force(&mut self, camera_force: CameraForce) {
        self.camera.apply_pitch(camera_force.pitch_radians);
        self.camera.apply_yaw(camera_force.yaw_radians);

        let camera_pos = self.camera.entity_fields.position;
        let feet_y = camera_pos.y - self.camera.height / 2.0;
        let head_y = camera_pos.y + self.camera.height / 2.0;

        // floor detection: downward ray from just above feet avoids false triggers
        // from walls and cube edges whose closest point happens to be at feet height
        const STEP: f32 = 0.1;
        let ray_origin = Vec3::new(camera_pos.x, feet_y + STEP, camera_pos.z);
        let on_ground = if let Some(hit) = self.intersect(ray_origin, Vec3::new(0.0, -1.0, 0.0)) {
            if hit.length_coefficient <= STEP * 2.0 && hit.surface_normal.y > 0.7 {
                let floor_y = ray_origin.y - hit.length_coefficient;
                let push_y = (floor_y + self.camera.height / 2.0) - camera_pos.y;
                if push_y > 0.0 {
                    self.camera.entity_fields.position.y += push_y;
                    self.camera.target.y += push_y;
                }
                if self.camera.entity_fields.velocity.y < 0.0 {
                    self.camera.entity_fields.velocity.y = 0.0;
                }
                true
            } else {
                false
            }
        } else {
            false
        };

        // horizontal (wall) collision: cylinder closest-point approach
        for entity in self.objects.values() {
            let closest = entity.closest_point(self.camera.entity_fields.position);
            let delta = self.camera.entity_fields.position - closest;

            let xz_distance = (delta.x * delta.x + delta.z * delta.z).sqrt();

            if closest.y > feet_y
                && closest.y < head_y
                && xz_distance < self.camera.radius
                && xz_distance > 0.0
            {
                let push_dir = Vec3::new(delta.x, 0.0, delta.z).normalize();
                let penetration = self.camera.radius - xz_distance;
                let push = push_dir.scalar_multiply(penetration);
                self.camera.entity_fields.position += push;
                self.camera.target += push;

                let inward = self.camera.entity_fields.velocity.dot(push_dir);
                if inward < 0.0 {
                    self.camera.entity_fields.velocity -= push_dir.scalar_multiply(inward);
                }
            }
        }

        let gravity = if on_ground {
            Vec3::ZERO
        } else {
            Vec3::new(0.0, -0.008, 0.0)
        };

        self.camera.entity_fields_mut().acceleration =
            camera_force.forces.scalar_multiply(INPUT_SCALE) + gravity;

        self.camera.update_target_and_forces();
    }

    /// Returns forward normal, right normal, up normal
    pub fn get_normals(&self) -> CameraNormals {
        CameraNormals {
            forward: self.camera.forward_normal(),
            right: self.camera.right_normal(),
            up: self.camera.up_normal(),
        }
    }

    pub fn get_camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    pub fn get_camera(&self) -> &Camera {
        &self.camera
    }
}
