use std::ops::{Deref, DerefMut};

use delve_shared::{
    math::{Mat3, Vec3},
    max, min,
};

pub trait Entity {
    /// Checks if there is a collision for this ray on this entity
    fn intersect(&self, ray_origin: Vec3, ray_direction: Vec3) -> Option<Collision>;

    /// Sets the gravity multiplier for the entity.
    /// <0.0 => negative gravity
    /// 0.0  => zero gravity
    /// 1.0  => normal gravity
    /// >1.0 => increased gravity
    fn set_gravity(&mut self, multiplier: f32);

    /// Sets the rotation matrix for the entity
    fn set_rotation(&mut self, rotation_matrix: Mat3);

    /// Set the position vector for the entity
    fn set_position(&mut self, position: Vec3);

    /// Set the velocity vector for the entity.
    /// This will in turn be applied to the position vector
    fn set_velocity(&mut self, velocity: Vec3);

    /// Add to the acceleration vector for this entity.
    /// This will in turn be applied to the velocity vector.
    /// The decay factor decays this acceleration vector on each application;
    /// close to 0.0 => decays very quickly
    /// close to 1.0 => decays very slowly
    fn add_acceleration(&mut self, acceleration: Vec3, decay: f32);

    /// clears all acceleration vectors
    fn clear_acceleration(&mut self);

    /// applies decay to acceleration, acceleration to velocity,
    /// then velocity to position.
    fn apply_forces(&mut self);
}

#[allow(unused)]
pub struct Collision {
    pub collision_position: Vec3,
    pub surface_normal: Vec3,
    pub length_coefficient: f32,
}

impl Collision {
    pub fn closest(a: Self, b: Self) -> Self {
        if a.length_coefficient < b.length_coefficient {
            return a;
        }

        b
    }
}

pub struct Sphere {
    acceleration: Vec<(Vec3, f32)>,
    velocity: Vec3,
    position: Vec3,
    radius: f32,
    gravity_multiplier: f32,
}

impl Sphere {
    pub fn new(position: Vec3, radius: f32) -> Self {
        Self {
            acceleration: Vec::new(),
            velocity: Vec3::ZERO,
            position,
            radius,
            gravity_multiplier: 1.0,
        }
    }
}

impl Entity for Sphere {
    /// The way this works is by solving for the points that the ray would
    /// collide with the sphere. In this case, we only return a collision
    /// if there are exactly two points that the ray collides with the sphere
    /// (entry and exit point), and those points are both positive (in front of the camera).
    /// the collision is at the closest point, the entry.
    /// ray_direction should be normalized
    fn intersect(&self, ray_origin: Vec3, ray_direction_normal: Vec3) -> Option<Collision> {
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

        let length_coefficient = min!(length_coefficient_one, length_coefficient_two);
        let collision_position =
            ray_origin + ray_direction_normal.scalar_multiply(length_coefficient);
        let surface_normal = (collision_position - self.position).normalize();

        Some(Collision {
            collision_position,
            surface_normal,
            length_coefficient,
        })
    }

    fn set_gravity(&mut self, multiplier: f32) {
        self.gravity_multiplier = multiplier;
    }

    fn set_rotation(&mut self, _rotation_matrix: Mat3) {
        () // does nothing for a sphere
    }

    fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }

    fn set_velocity(&mut self, velocity: Vec3) {
        self.velocity = velocity;
    }

    fn add_acceleration(&mut self, acceleration: Vec3, decay: f32) {
        self.acceleration.push((acceleration, decay));
    }

    fn clear_acceleration(&mut self) {
        self.acceleration.clear();
    }

    fn apply_forces(&mut self) {
        // TODO: this is expensive because we heap alloc each time. would this be better with a
        // hash set?
        let mut new_accel = Vec::with_capacity(self.acceleration.len());

        for (accel, decay) in self.acceleration.iter_mut() {
            *accel = accel.scalar_multiply(*decay);

            self.velocity += *accel;
            self.position += self.velocity;

            // this is bad
            if accel.square_magnitude() > 0.01 {
                new_accel.push((accel.clone(), decay.clone()));
            }
        }

        self.acceleration = new_accel;
    }
}

pub struct Cube(RectPrism);

impl Cube {
    pub fn new(height: f32, center: Vec3) -> Self {
        Self(RectPrism::new(Vec3::new(height, height, height), center))
    }

    pub fn sin_hover(&mut self, start: std::time::Instant) {
        let theta = start.elapsed().as_secs_f32();

        self.0.center.y = 0.3 * ((theta * 2.0).sin());

        self.set_rotation(Mat3::from_axis_angle(Vec3::Y, theta));
    }
}

impl Deref for Cube {
    type Target = RectPrism;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Cube {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct RectPrism {
    /// NOT column-vector form; storing the transpose
    /// [[right], [up], [forward]]
    rotation: Mat3,
    /// width, height, length
    dimensions: Vec3,
    center: Vec3,
}

impl RectPrism {
    pub fn new(dimensions: Vec3, center: Vec3) -> Self {
        Self {
            rotation: Mat3::identity(),
            dimensions,
            center,
        }
    }

    pub fn set_rotation(&mut self, r: Mat3) {
        self.rotation = r;
    }

    /// slab method; axis-aligned cube
    /// for each axis, solve for the point on the ray where that axis' coordinate
    /// is equal to center +- height/2
    /// cube = 3 sets of parallel, axis-aligned planes
    pub fn intersect(&self, ray_origin: Vec3, ray_direction: Vec3) -> Option<Collision> {
        let rotation_transpose: Mat3 = self.rotation.transpose().into();

        // transform into local coordinate space
        let ray_origin = rotation_transpose.apply(ray_origin - self.center);
        let ray_direction = rotation_transpose.apply(ray_direction);

        let hx = self.dimensions.x / 2.0;
        let t1 = (-hx - ray_origin.x) / ray_direction.x;
        let t2 = (hx - ray_origin.x) / ray_direction.x;
        let t_enter_x = min!(t1, t2);
        let t_exit_x = max!(t1, t2);

        let hy = self.dimensions.y / 2.0;
        let t1 = (-hy - ray_origin.y) / ray_direction.y;
        let t2 = (hy - ray_origin.y) / ray_direction.y;
        let t_enter_y = min!(t1, t2);
        let t_exit_y = max!(t1, t2);

        let hz = self.dimensions.z / 2.0;
        let t1 = (-hz - ray_origin.z) / ray_direction.z;
        let t2 = (hz - ray_origin.z) / ray_direction.z;
        let t_enter_z = min!(t1, t2);
        let t_exit_z = max!(t1, t2);

        let t_enter = max!(t_enter_x, t_enter_y, t_enter_z);
        let t_exit = min!(t_exit_x, t_exit_y, t_exit_z);

        if t_enter <= t_exit && t_exit > 0.0 {
            let mut position = ray_origin + ray_direction.scalar_multiply(t_enter);

            let mut normal = Vec3::ORIGIN;
            if t_enter_x == t_enter {
                normal.x = 1.0 * -ray_direction.x.signum();
            } else if t_enter_y == t_enter {
                normal.y = 1.0 * -ray_direction.y.signum();
            } else {
                normal.z = 1.0 * -ray_direction.z.signum();
            }

            // we are still in local AABB coordinate space so now we need to rotate out
            // inverse == transpose
            position = self.rotation.apply(position) + self.center;
            normal = self.rotation.apply(normal);

            return Some(Collision {
                collision_position: position,
                length_coefficient: t_enter,
                surface_normal: normal,
            });
        }

        None
    }
}

#[allow(unused)]
pub enum Shape {
    Sphere(Sphere),
    RectPrism(RectPrism),
    Cube(Cube),
}
