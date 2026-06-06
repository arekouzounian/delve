/// I know you can do some crazy stuff with D&C and SIMD but I'm
/// optimizing for my sanity over performance
///
/// Static matrix and vector implementations; no heap allocations.
/// we can kinda get away with this by encoding the dimensions into the
/// type parameters which is kinda nuts
use std::fmt::Debug;
use std::ops::{Add, AddAssign, Mul, Sub, SubAssign, Deref, DerefMut};

use num_traits::NumAssign;
use rand::prelude::*;

use crate::mat;

pub trait NumericType: NumAssign + Copy + Clone + Debug {}
impl<T: NumAssign + Copy + Clone + Debug> NumericType for T {}

#[derive(Debug, Clone, PartialEq)]
pub struct Matrix<T, const R: usize, const C: usize> {
    /// inner.len() == m == rows
    /// inner[0].len() == n == cols
    inner: [[T; C]; R],
}

/// iterate through all rows
#[derive(Debug, Clone)]
pub struct Rows<'a, T, const R: usize, const C: usize> {
    inner: &'a Matrix<T, R, C>,
    row: usize,
}

/// iterate through all columns
#[derive(Debug, Clone)]
pub struct Cols<'a, T, const R: usize, const C: usize> {
    inner: &'a Matrix<T, R, C>,
    col: usize,
}

/// iterate through elements of a single row
#[derive(Debug, Clone)]
pub struct RowIterator<'a, T, const R: usize, const C: usize> {
    inner: &'a Matrix<T, R, C>,
    row: usize,
    col: usize,
}

/// iterate through elements of a single column
#[derive(Debug, Clone)]
pub struct ColIterator<'a, T, const R: usize, const C: usize> {
    inner: &'a Matrix<T, R, C>,
    row: usize,
    col: usize,
}

/// iterator over the rows
impl<'a, T, const R: usize, const C: usize> Iterator for Rows<'a, T, R, C> {
    type Item = RowIterator<'a, T, R, C>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.row >= R {
            return None;
        }

        let next = RowIterator {
            inner: self.inner,
            row: self.row,
            col: 0,
        };
        self.row += 1;

        Some(next)
    }
}

impl<'a, T, const R: usize, const C: usize> Iterator for Cols<'a, T, R, C> {
    type Item = ColIterator<'a, T, R, C>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.col >= C {
            return None;
        }

        let next = ColIterator {
            inner: self.inner,
            row: 0,
            col: self.col,
        };
        self.col += 1;

        Some(next)
    }
}

impl<'a, T, const R: usize, const C: usize> Iterator for RowIterator<'a, T, R, C> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.inner.inner.get(self.row)?.get(self.col)?;
        self.col += 1;

        Some(next)
    }
}

impl<'a, T, const R: usize, const C: usize> Iterator for ColIterator<'a, T, R, C> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.inner.inner.get(self.row)?.get(self.col)?;
        self.row += 1;

        Some(next)
    }
}

impl<T, const R: usize, const C: usize> Matrix<T, R, C>
where
    T: NumericType,
{
    pub fn zero() -> Self {
        Self {
            inner: [[T::zero(); C]; R],
        }
    }

    pub fn identity() -> Matrix<f32, R, C> {
        let mut m = Matrix::zero();

        for r in 0..R {
            for c in 0..C {
                if r == c {
                    m.inner[r][c] = 1.0f32;
                }
            }
        }

        m
    }

    pub fn new(inner: [[T; C]; R]) -> Self {
        Self { inner }
    }

    pub fn rows<'a>(&'a self) -> Rows<'a, T, R, C> {
        Rows {
            inner: self,
            row: 0,
        }
    }

    pub fn cols<'a>(&'a self) -> Cols<'a, T, R, C> {
        Cols {
            inner: self,
            col: 0,
        }
    }

    pub fn scalar_multiply(&mut self, scalar: T) {
        for row in self.inner.iter_mut() {
            for col in row.iter_mut() {
                *col *= scalar;
            }
        }
    }

    pub fn transpose(&self) -> Matrix<T, C, R> {
        let mut transpose = Matrix::zero();

        for r in 0..R {
            for c in 0..C {
                transpose.inner[c][r] = self.inner[r][c];
            }
        }

        transpose
    }

    /// will panic if i > cols
    pub fn col(&self, i: usize) -> Matrix<T, R, 1> {
        let mut m = Matrix::zero();

        for j in 0..C {
            m.inner[j][0] = self.inner[i][j];
        }

        m
    }

    /// will panic if i > rows
    pub fn row(&self, i: usize) -> Matrix<T, 1, C> {
        let mut m = Matrix::zero();

        for j in 0..R {
            m.inner[0][j] = self.inner[j][i];
        }

        m
    }
}

impl<T, const R: usize, const C: usize> Add for Matrix<T, R, C>
where
    T: NumericType,
{
    type Output = Self;

    fn add(mut self, other: Self) -> Self::Output {
        for row in 0..R {
            for col in 0..C {
                self.inner[row][col] += other.inner[row][col];
            }
        }

        self
    }
}

impl<T, const R: usize, const C: usize> Sub for Matrix<T, R, C>
where
    T: NumericType,
{
    type Output = Self;

    fn sub(mut self, other: Self) -> Self::Output {
        for row in 0..R {
            for col in 0..C {
                self.inner[row][col] -= other.inner[row][col];
            }
        }

        self
    }
}

/// A m x n matrix can only be multiplied by an n x p matrix
/// the result is a m x p matrix
impl<T, const M: usize, const N: usize, const P: usize> Mul<Matrix<T, N, P>> for Matrix<T, M, N>
where
    T: NumericType,
{
    type Output = Matrix<T, M, P>;

    fn mul(self, other: Matrix<T, N, P>) -> Self::Output {
        let mut multiplied_matrix: Matrix<T, M, P> = Matrix::zero();

        for i in 0..M {
            for j in 0..P {
                let mut sum = T::zero();

                for k in 0..N {
                    sum += self.inner[i][k] * other.inner[k][j];
                }

                multiplied_matrix.inner[i][j] = sum;
            }
        }

        multiplied_matrix
    }
}

pub struct Mat3(Matrix<f32, 3, 3>);

impl Mat3 {
    pub fn zero() -> Self {
        Self(Matrix::<f32, 3, 3>::zero())
    }

    pub fn identity() -> Self {
        Self(Matrix::<f32, 3, 3>::identity())
    }

    /// ive been trying to do this through the type system but its too much of a mess.
    /// TODO: can we make this just use the matrix mult math without having extra allocs
    /// from type conversions
    /// idk look at this later it's probably horribly inefficient
    pub fn apply(&self, rhs: Vec3) -> Vec3 {
        Vec3::new(
            Vec3::from(self.0.row(0)).dot(rhs),
            Vec3::from(self.0.row(1)).dot(rhs),
            Vec3::from(self.0.row(2)).dot(rhs),
        )
    }
}

impl From<Matrix<f32, 3, 3>> for Mat3 {
    fn from(rhs: Matrix<f32, 3, 3>) -> Self {
        Self(rhs)
    }
}

impl From<Mat3> for Matrix<f32, 3, 3> {
    fn from(rhs: Mat3) -> Matrix<f32, 3, 3> {
        rhs.0
    }
}

impl Deref for Mat3 {
    type Target = Matrix<f32, 3, 3>;

    fn deref(&self) -> &Matrix<f32, 3, 3> {
        &self.0
    }
}

impl DerefMut for Mat3 {
    fn deref_mut(&mut self) -> &mut Matrix<f32, 3, 3> {
        &mut self.0
    }

}

// TODO: refactor using mat
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
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

    #[allow(unused)]
    pub fn from_random(rng: &mut ThreadRng) -> Self {
        Self {
            x: rng.random(),
            y: rng.random(),
            z: rng.random(),
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

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
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

impl SubAssign for Vec3 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl<T> From<Matrix<T, 3, 1>> for Vec3
where T: NumericType + Into<f32>
{
    fn from(other: Matrix<T, 3, 1>) -> Self {
        Vec3::new(
            other.inner[0][0].into(),
            other.inner[1][0].into(),
            other.inner[2][0].into(),
        )
    }
}

impl<T> From<Matrix<T, 1, 3>> for Vec3 
where T: NumericType + Into<f32>
{
    fn from(other: Matrix<T, 1, 3>) -> Self {
        Vec3::new(
            other.inner[0][0].into(),
            other.inner[0][1].into(),
            other.inner[0][2].into(),
        )
    }
}

impl<T> From<Vec3> for Matrix<T, 3, 1>
where T: NumericType + From<f32>
{
    fn from(other: Vec3) -> Self {
        mat![
            [T::from(other.x)],
            [T::from(other.y)],
            [T::from(other.z)],
        ]
    }
}


#[cfg(test)]
mod tests {
    #[allow(unused)]
    use super::*;
    use crate::mat;

    #[test]
    fn test_matrix_multiplication() {
        let a = mat![[4, 2, 0], [0, 6, 9]];
        let b = mat![[12, 8], [4, 9], [3, 7]];

        let result = mat![[56, 50], [51, 117]];

        assert!(a * b == result);
    }

    #[test]
    fn test_matrix_addition() {
        let a = mat![[1, 2, 3, 4, 5], [6, 7, 8, 9, 10]];
        let b = mat![[2, 4, 6, 8, 10], [12, 14, 16, 18, 20]];

        let result = mat![[3, 6, 9, 12, 15], [18, 21, 24, 27, 30]];

        assert!(a + b == result);
    }

    #[test]
    fn test_transpose() {
        let m = mat![[1, 2, 3], [4, 5, 6]];
        let expected = mat![
            [1, 4],
            [2, 5],
            [3, 6]
        ];

        assert!(m.transpose() == expected);
    }

    #[test]
    fn test_conversion() {
        let m = mat![[1.0], [2.0], [3.0]];
        let v = Vec3::new(1.0, 2.0, 3.0);

        assert!(Vec3::from(m) == v);
    }

    #[test]
    fn test_conversion_with_differing_types() {
        let m = mat![[1u16], [2u16], [3u16]];
        let v = Vec3::new(1.0, 2.0, 3.0);

        let vec = Vec3::from(m);

        assert!(vec == v);
    }
}
