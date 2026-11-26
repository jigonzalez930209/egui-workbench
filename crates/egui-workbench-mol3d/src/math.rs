//! A small vector and quaternion layer for the camera.

/// A 3D vector.
#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Vec3 {
    /// x.
    pub x: f64,
    /// y.
    pub y: f64,
    /// z.
    pub z: f64,
}

impl Vec3 {
    /// The zero vector.
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);

    /// A vector.
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Dot product.
    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Cross product.
    pub fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    /// Euclidean length.
    pub fn length(self) -> f64 {
        self.dot(self).sqrt()
    }

    /// Unit vector, or zero for a zero-length input.
    pub fn normalized(self) -> Self {
        let length = self.length();
        if length <= f64::EPSILON {
            Self::ZERO
        } else {
            self * (1.0 / length)
        }
    }

    /// Component-wise finite check.
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }

    /// Linear interpolation.
    pub fn lerp(self, other: Self, t: f64) -> Self {
        self + (other - self) * t
    }
}

impl std::ops::Add for Vec3 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl std::ops::Sub for Vec3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl std::ops::Mul<f64> for Vec3 {
    type Output = Self;

    fn mul(self, scale: f64) -> Self {
        Self::new(self.x * scale, self.y * scale, self.z * scale)
    }
}

impl std::ops::Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }
}

/// A unit quaternion.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Quat {
    /// x.
    pub x: f64,
    /// y.
    pub y: f64,
    /// z.
    pub z: f64,
    /// w.
    pub w: f64,
}

impl Default for Quat {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Quat {
    /// The identity rotation.
    pub const IDENTITY: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 1.0,
    };

    /// A rotation of `angle` radians around `axis`.
    pub fn from_axis_angle(axis: Vec3, angle: f64) -> Self {
        let axis = axis.normalized();
        if !axis.is_finite() || axis.length() <= f64::EPSILON {
            return Self::IDENTITY;
        }
        let half = angle * 0.5;
        let sine = half.sin();
        Self {
            x: axis.x * sine,
            y: axis.y * sine,
            z: axis.z * sine,
            w: half.cos(),
        }
    }

    /// The inverse rotation.
    pub fn conjugate(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: self.w,
        }
    }

    /// Normalize the quaternion.
    pub fn normalized(self) -> Self {
        let norm = (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt();
        if norm <= f64::EPSILON {
            Self::IDENTITY
        } else {
            Self {
                x: self.x / norm,
                y: self.y / norm,
                z: self.z / norm,
                w: self.w / norm,
            }
        }
    }

    /// Rotate a vector.
    pub fn rotate(self, vector: Vec3) -> Vec3 {
        let q = self.normalized();
        let uv = Vec3::new(q.x, q.y, q.z).cross(vector);
        vector + uv * (2.0 * q.w) + Vec3::new(q.x, q.y, q.z).cross(uv) * 2.0
    }

    /// The camera basis: right, up, and backward (`-forward`).
    pub fn basis(self) -> (Vec3, Vec3, Vec3) {
        (
            self.rotate(Vec3::new(1.0, 0.0, 0.0)),
            self.rotate(Vec3::new(0.0, 1.0, 0.0)),
            self.rotate(Vec3::new(0.0, 0.0, 1.0)),
        )
    }

    /// Component-wise finite check.
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite() && self.w.is_finite()
    }
}

impl std::ops::Mul for Quat {
    type Output = Self;

    /// Hamilton product: `self` applied after `other`.
    fn mul(self, other: Self) -> Self {
        Self {
            w: self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z,
            x: self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y,
            y: self.w * other.y - self.x * other.z + self.y * other.w + self.z * other.x,
            z: self.w * other.z + self.x * other.y - self.y * other.x + self.z * other.w,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Mul;

    #[test]
    fn quaternion_rotates_a_quarter_turn() {
        let rotation = Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), std::f64::consts::FRAC_PI_2);
        let rotated = rotation.rotate(Vec3::new(1.0, 0.0, 0.0));
        assert!((rotated.x).abs() < 1e-9);
        assert!((rotated.y - 1.0).abs() < 1e-9);
    }

    #[test]
    fn a_rotation_keeps_the_basis_orthonormal() {
        let rotation = Quat::from_axis_angle(Vec3::new(0.3, 0.7, 0.2), 1.1)
            .mul(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), -0.4));
        let (right, up, back) = rotation.basis();
        assert!((right.length() - 1.0).abs() < 1e-9);
        assert!((up.length() - 1.0).abs() < 1e-9);
        assert!((back.length() - 1.0).abs() < 1e-9);
        assert!(right.dot(up).abs() < 1e-9);
        assert!(right.dot(back).abs() < 1e-9);
    }

    #[test]
    fn rotating_then_inverse_restores_the_vector() {
        let rotation = Quat::from_axis_angle(Vec3::new(1.0, -2.0, 0.5), 0.9);
        let vector = Vec3::new(3.0, -1.0, 2.0);
        let round_trip = rotation.conjugate().rotate(rotation.rotate(vector));
        assert!((round_trip - vector).length() < 1e-9);
    }
}
