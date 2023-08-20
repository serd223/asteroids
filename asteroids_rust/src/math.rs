use std::ops::{Add, AddAssign, Mul, MulAssign};

#[derive(Default, Clone)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

/// Wrapper for Vec2::new
pub fn vec2(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

impl Mul<f32> for Vec2 {
    type Output = Self;

    fn mul(mut self, rhs: f32) -> Self::Output {
        self.x *= rhs;
        self.y *= rhs;
        self
    }
}

impl MulAssign<f32> for Vec2 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl Add for Vec2 {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.x += rhs.x;
        self.y += rhs.y;
        self
    }
}

impl Add<&Vec2> for Vec2 {
    type Output = Self;

    fn add(mut self, rhs: &Self) -> Self::Output {
        self.x += rhs.x;
        self.y += rhs.y;
        self
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl AddAssign<&Vec2> for Vec2 {
    fn add_assign(&mut self, rhs: &Vec2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Vec2 {
    pub const ZERO: Self = Vec2 { x: 0., y: 0. };

    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn rotate(mut self, rot: f32) -> Self {
        self.rotate_mut(rot);
        self
    }

    pub fn rotate_mut(&mut self, rot: f32) {
        let (s, c) = rot.sin_cos();
        let x = self.x * c - self.y * s;
        let y = self.y * c + self.x * s;
        self.x = x;
        self.y = y;
    }

    pub fn normalise(mut self) -> Self {
        self.normalise_mut();
        self
    }

    pub fn normalise_mut(&mut self) {
        let len = (self.x * self.x + self.y * self.y).sqrt();

        self.x /= len;
        self.y /= len;
    }
}

#[derive(Clone)]
pub struct Transform<const N: usize> {
    pub pos: Vec2,
    pub vertices: [Vec2; N],
    pub transform: [Vec2; N],
    pub scale: f32,
    pub rot: f32,
}

impl<const N: usize> Transform<N> {
    pub fn apply(&mut self) {
        for (i, v) in self.vertices.iter().enumerate() {
            self.transform[i] = v.clone().rotate(self.rot) * self.scale;
            self.transform[i].x += self.pos.x;
            self.transform[i].y += self.pos.y;
        }
    }
}
