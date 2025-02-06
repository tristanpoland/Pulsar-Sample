pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Vector3 { x, y, z }
    }

    pub fn zero() -> Self {
        Vector3::new(0.0, 0.0, 0.0)
    }

    pub fn one() -> Self {
        Vector3::new(1.0, 1.0, 1.0)
    }

    pub fn up() -> Self {
        Vector3::new(0.0, 1.0, 0.0)
    }

    pub fn down() -> Self {
        Vector3::new(0.0, -1.0, 0.0)
    }

    pub fn left() -> Self {
        Vector3::new(-1.0, 0.0, 0.0)
    }

    pub fn right() -> Self {
        Vector3::new(1.0, 0.0, 0.0)
    }

    pub fn forward() -> Self {
        Vector3::new(0.0, 0.0, 1.0)
    }

    pub fn back() -> Self {
        Vector3::new(0.0, 0.0, -1.0)
    }

    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();
        Vector3::new(self.x / mag, self.y / mag, self.z / mag)
    }

    pub fn dot(&self, other: &Vector3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: &Vector3) -> Vector3 {
        Vector3::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    pub fn lerp(&self, other: &Vector3, t: f32) -> Vector3 {
        Vector3::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
            self.z + (other.z - self.z) * t,
        )
    }

    pub fn distance(&self, other: &Vector3) -> f32 {
        (self - other).magnitude()
    }

    pub fn angle(&self, other: &Vector3) -> f32 {
        let dot = self.dot(other);
        let mag = self.magnitude() * other.magnitude();
        dot.acos() / mag
    }

    pub fn reflect(&self, normal: &Vector3) -> Vector3 {
        self - normal * 2.0 * self.dot(normal)
    }

    pub fn transform(&self, matrix: &Matrix4) -> Vector3 {
        Vector3::new(
            self.x * matrix.m11 + self.y * matrix.m21 + self.z * matrix.m31 + matrix.m41,
            self.x * matrix.m12 + self.y * matrix.m22 + self.z * matrix.m32 + matrix.m42,
            self.x * matrix.m13 + self.y * matrix.m23 + self.z * matrix.m33 + matrix.m43,
        )
    }

    pub fn transform_normal(&self, matrix: &Matrix4) -> Vector3 {
        Vector3::new(
            self.x * matrix.m11 + self.y * matrix.m21 + self.z * matrix.m31,
            self.x * matrix.m12 + self.y * matrix.m22 + self.z * matrix.m32,
            self.x * matrix.m13 + self.y * matrix.m23 + self.z * matrix.m33,
        )
    }

    pub fn transform_direction(&self, matrix: &Matrix4) -> Vector3 {
        self.normalize().transform_normal(matrix)
    }

    pub fn transform_position(&self, matrix: &Matrix4) -> Vector3 {
        Vector3::new(
            self.x * matrix.m11 + self.y * matrix.m21 + self.z * matrix.m31 + matrix.m41,
            self.x * matrix.m12 + self.y * matrix.m22 + self.z * matrix.m32 + matrix.m42,
            self.x * matrix.m13 + self.y * matrix.m23 + self.z * matrix.m33 + matrix.m43,
        )
    }

    pub fn transform_vector(&self, matrix: &Matrix4) -> Vector3 {
        Vector3::new(
            self.x * matrix.m11 + self.y * matrix.m21 + self.z * matrix.m31,
            self.x * matrix.m12 + self.y * matrix.m22 + self.z * matrix.m32,
            self.x * matrix.m13 + self.y * matrix.m23 + self.z * matrix.m33,
        )
    }

    pub fn transform_direction(&self, matrix: &Matrix4) -> Vector3 {
        self.normalize().transform_vector(matrix)
    }
}