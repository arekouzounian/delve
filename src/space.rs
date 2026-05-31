use crate::util::min;
use crate::vector::Vec3;

pub struct Camera {
    position: Vec3,
    target: Vec3,
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
        Vec3::cross_product(&self.forward_normal(), &Self::WORLD_UP).normalize()
    }

    pub fn up_normal(&self) -> Vec3 {
        Vec3::cross_product(&self.right_normal(), &self.forward_normal())
    }
}

pub struct Sphere {
    position: Vec3,
    radius: f32,
}

pub struct Collision {
    collision_position: Vec3,
    surface_normal: Vec3,

    // origin point of the ray + length_coefficient * direction vector
    // this gives us the position of the hit point
    // do we need this?
    length_coefficient: f32,
}

impl Sphere {
    pub fn new(position: Vec3, radius: f32) -> Self {
        Self { position, radius }
    }

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

pub struct Scene {
    pub camera: Camera,
    objects: Vec<Shape>,
}

impl Scene {
    /// fov_rad: field of view angle, in radians
    pub fn new(camera: Camera) -> Self {
        Self {
            camera,
            objects: Vec::new(),
        }
    }

    // TODO: object de-registration/ID tracking
    pub fn register_shape(&mut self, shape: Shape) {
        self.objects.push(shape);
    }

    // TODO: naively checks all shapes; should optimize this
    pub fn intersect(&mut self, ray_origin: Vec3, ray_direction: Vec3) -> Option<Collision> {
        for shape in &self.objects {
            let result = match shape {
                Shape::Sphere(s) => s.intersect(ray_origin, ray_direction),
            };

            if result.is_some() {
                return result;
            }
        }

        None
    }
}

