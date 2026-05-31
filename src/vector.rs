use std::ops::{Add, Sub};

#[derive(Debug, Clone, Copy)]
pub struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    pub const ORIGIN: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    // a = a_1(x) + a_2(y) + a_3(z)
    // b = b_1(x) + b_2(y) + b_3(z)
    // a x b = (a_2b_3 - a_3b_2)(x) + (a_3b_1 - a_1b_3)(y) + (a_1b_2 - a_2b_1)(z)
    // https://en.wikipedia.org/wiki/Cross_product
    pub fn cross_product(a: &Vec3, b: &Vec3) -> Self {
        Self {
            x: (a.y * b.z) - (a.z * b.y),
            y: (a.z * b.x) - (a.x * b.z),
            z: (a.x * b.y) - (a.y * b.x),
        }
    }

    pub fn scalar_multiply(&self, scalar: f32) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }

    // a * b = a_1b_1 + a_2b_2 + a_3b_3
    pub fn dot_product(a: Vec3, b: Vec3) -> f32 {
        (a.x * b.x) + (a.y * b.y) + (a.z * b.z)
    }

    pub fn dot(self, other: Self) -> f32 {
        Vec3::dot_product(self, other)
    }

    pub fn normalize(self) -> Self {
        let magnitude = ((self.x * self.x) + (self.y * self.y) + (self.z * self.z)).sqrt();

        Self {
            x: self.x / magnitude,
            y: self.y / magnitude,
            z: self.z / magnitude,
        }
    }
}

impl Add for Vec3 {
    type Output = Vec3;

    fn add(self, other: Vec3) -> Vec3 {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}
