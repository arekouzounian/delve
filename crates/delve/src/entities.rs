use std::ops::{Deref, DerefMut};

use delve_shared::{
    math::{Mat3, Vec3},
    max, min,
    traits::Entity,
    types::Collision,
};

use delve_macros::entity;

pub enum Entities {
    Sphere(Sphere),
    RectPrism(RectPrism),
    Cube(Cube),
    Plane(Plane),
}

impl Entities {
    pub fn intersect(&self, ray_origin: Vec3, ray_direction: Vec3) -> Option<Collision> {
        match self {
            Self::Sphere(s) => s.intersect(ray_origin, ray_direction),
            Self::RectPrism(r) => r.intersect(ray_origin, ray_direction),
            Self::Cube(c) => c.intersect(ray_origin, ray_direction),
            Self::Plane(p) => p.intersect(ray_origin, ray_direction),
        }
    }

    pub fn closest_point(&self, origin_point: Vec3) -> Vec3 {
        match self {
            Self::Sphere(s) => s.closest_point(origin_point),
            Self::RectPrism(r) => r.closest_point(origin_point),
            Self::Cube(c) => c.closest_point(origin_point),
            Self::Plane(p) => p.closest_point(origin_point),
        }
    }
}

#[entity]
pub struct Sphere {
    radius: f32,
}

impl Sphere {
    pub fn new(radius: f32) -> Self {
        Self {
            radius,
            ..Self::default()
        }
    }

    /// The way this works is by solving for the points that the ray would
    /// collide with the sphere. In this case, we only return a collision
    /// if there are exactly two points that the ray collides with the sphere
    /// (entry and exit point), and those points are both positive (in front of the camera).
    /// the collision is at the closest point, the entry.
    /// ray_direction should be normalized
    fn intersect(&self, ray_origin: Vec3, ray_direction_normal: Vec3) -> Option<Collision> {
        if self.entity_fields.invisible {
            return None;
        }

        let origin_center_diff = ray_origin - self.entity_fields.position;

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
        let surface_normal = (collision_position - self.entity_fields.position).normalize();

        Some(Collision {
            collision_position,
            surface_normal,
            length_coefficient,
        })
    }

    fn closest_point(&self, origin_point: Vec3) -> Vec3 {
        (origin_point - self.entity_fields.position)
            .normalize()
            .scalar_multiply(self.radius)
            + self.entity_fields.position
    }
}

pub struct Cube(RectPrism);

impl Cube {
    pub fn new(height: f32) -> Self {
        Self(RectPrism::new(Vec3::new(height, height, height)))
    }

    pub fn sin_hover(&mut self, start: std::time::Instant) {
        let theta = start.elapsed().as_secs_f32();

        self.entity_fields_mut().position.y = 0.3 * ((theta * 2.0).sin());

        self.entity_fields_mut().rotation = Mat3::from_axis_angle(Vec3::Y, theta);
    }

    pub fn length(&self) -> f32 {
        self.0.dimensions.x
    }
}

impl Entity for Cube {
    fn entity_fields(&self) -> &delve_shared::types::EntityFields {
        self.0.entity_fields()
    }

    fn entity_fields_mut(&mut self) -> &mut delve_shared::types::EntityFields {
        self.0.entity_fields_mut()
    }

    fn apply_forces(&mut self) {
        self.0.apply_forces();
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

#[entity]
pub struct RectPrism {
    /// width, height, length
    dimensions: Vec3,
}

impl RectPrism {
    pub fn new(dimensions: Vec3) -> Self {
        Self {
            dimensions,
            ..Self::default()
        }
    }

    pub fn length(&self) -> f32 {
        self.dimensions.x
    }

    pub fn width(&self) -> f32 {
        self.dimensions.z
    }

    pub fn height(&self) -> f32 {
        self.dimensions.y
    }

    /// slab method; axis-aligned cube
    /// for each axis, solve for the point on the ray where that axis' coordinate
    /// is equal to center +- height/2
    /// cube = 3 sets of parallel, axis-aligned planes
    pub fn intersect(&self, ray_origin: Vec3, ray_direction: Vec3) -> Option<Collision> {
        if self.entity_fields.invisible {
            return None;
        }

        let rotation_transpose: Mat3 = self.entity_fields.rotation.transpose().into();

        // transform into local coordinate space
        let ray_origin = rotation_transpose.apply(ray_origin - self.entity_fields.position);
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
            position = self.entity_fields.rotation.apply(position) + self.entity_fields.position;
            normal = self.entity_fields.rotation.apply(normal);

            return Some(Collision {
                collision_position: position,
                length_coefficient: t_enter,
                surface_normal: normal,
            });
        }

        None
    }

    fn closest_point(&self, origin_point: Vec3) -> Vec3 {
        let rotation_transpose: Mat3 = self.entity_fields.rotation.transpose().into();
        let mut op_local = rotation_transpose.apply(origin_point - self.entity_fields.position);

        // clamp it to within our prism dimensions
        let x_half_extent = self.dimensions.x / 2.0;
        let y_half_extent = self.dimensions.y / 2.0;
        let z_half_extent = self.dimensions.z / 2.0;
        op_local.x = op_local.x.clamp(-x_half_extent, x_half_extent);
        op_local.y = op_local.y.clamp(-y_half_extent, y_half_extent);
        op_local.z = op_local.z.clamp(-z_half_extent, z_half_extent);

        // transform back into world space
        self.entity_fields.rotation.apply(op_local) + self.entity_fields.position
    }
}

/// in theory should be infinite, but we would need some sort of dropoff calculation to avoid
/// infinitely checking ray collisions
// TODO: we can get away with not storing normals & just using the rotation matrix i think
#[entity]
pub struct Plane {
    width: f32,
    length: f32,
    /// world normal, not local
    normal: Vec3,
}

impl Plane {
    pub fn new(width: f32, length: f32, normal: Vec3) -> Self {
        Self {
            width,
            length,
            normal,
            ..Self::default()
        }
    }

    fn intersect(&self, ray_origin: Vec3, ray_direction: Vec3) -> Option<Collision> {
        if self.entity_fields.invisible {
            return None;
        }

        let rotation_transpose: Mat3 = self.entity_fields.rotation.transpose().into();

        // transform into local coordinate space
        let ray_origin = rotation_transpose.apply(ray_origin - self.entity_fields.position);
        let ray_direction = rotation_transpose.apply(ray_direction);
        let normal = rotation_transpose.apply(self.normal);

        // P(t) = ray_origin + (t * ray_direction); this yields a point on the ray
        // Q = plane center, N = plane normal
        // the plane is defined as some point Q and some normal N
        // for any point P, if (P - Q) * N = 0, that means the vector from Q to P is on the plane,
        // which means P is on the plane.
        let denom = normal.dot(ray_direction);
        let numer = -normal.dot(ray_origin); // since we're in local space, plane is at the origin

        if denom.abs() < 1e-6 {
            return None;
        }

        let t = numer / denom; // is div by zero possible?
        if t < 0.0 {
            return None;
        }

        let collision = ray_origin + ray_direction.scalar_multiply(t);

        if collision.x.abs() > self.width / 2.0 || collision.z.abs() > self.length / 2.0 {
            return None;
        }

        Some(Collision {
            surface_normal: self.normal,
            collision_position: self.entity_fields.rotation.apply(collision)
                + self.entity_fields.position,
            length_coefficient: t,
        })
    }

    fn closest_point(&self, origin_point: Vec3) -> Vec3 {
        let rotation_transpose: Mat3 = self.entity_fields.rotation.transpose().into();
        let mut op_local = rotation_transpose.apply(origin_point - self.entity_fields.position);
        let local_normal = rotation_transpose.apply(self.normal);
        op_local -= local_normal.scalar_multiply(op_local.dot(local_normal));

        // clamp it to within our prism dimensions
        let x_half_extent = self.width / 2.0;
        let z_half_extent = self.length / 2.0;
        op_local.x = op_local.x.clamp(-x_half_extent, x_half_extent);
        op_local.z = op_local.z.clamp(-z_half_extent, z_half_extent);

        // transform back into world space
        self.entity_fields.rotation.apply(op_local) + self.entity_fields.position
    }
}
