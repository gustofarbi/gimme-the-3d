use std::fmt;
use std::ops::Mul;

use cgmath::InnerSpace;
use cgmath::SquareMatrix;
use nalgebra::{Matrix4, Point3, Quaternion, Rotation3, UnitQuaternion};
use three_d::Vector4;
use three_d_asset::Mat4;

#[derive(Debug, Clone)]
pub struct Camera {
    pub parent_transform: Transform,
    pub transform: Transform,
    pub aspect_ratio: f32,
    pub yfov: f32,
    pub zfar: f32,
    pub znear: f32,
}

#[derive(Debug)]
pub struct Mesh {
    pub parent_transform: Transform,
    pub transform: Transform,
}

#[derive(Clone, Copy)]
pub struct Transform {
    pub matrix: Matrix4<f32>,
}

fn float_eq(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.0001
}

impl Transform {
    pub fn has_equal_rotation(&self, other: &Self) -> bool {
        let (_, r1, _) = self.decomposed();
        let (_, r2, _) = other.decomposed();
        float_eq(r1[0], r2[0])
            && float_eq(r1[1], r2[1])
            && float_eq(r1[2], r2[2])
            && float_eq(r1[3], r2[3])
    }

    pub fn from_quaternion(quaternion: Quaternion<f32>) -> Self {
        let t = gltf::scene::Transform::Decomposed {
            translation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            rotation: [
                quaternion.coords.x,
                quaternion.coords.y,
                quaternion.coords.z,
                quaternion.coords.w,
            ],
        };
        Self::from(t)
    }

    pub fn decomposed(&self) -> ([f32; 3], [f32; 4], [f32; 3]) {
        let translation = [self.matrix.m41, self.matrix.m42, self.matrix.m43];
        #[rustfmt::skip]
            let mut i = cgmath::Matrix3::new(
            self.matrix.m11, self.matrix.m21, self.matrix.m31,
            self.matrix.m12, self.matrix.m22, self.matrix.m32,
            self.matrix.m13, self.matrix.m23, self.matrix.m33,
        );

        let sx = i.x.magnitude();
        let sy = i.y.magnitude();
        let sz = i.determinant().signum() * i.z.magnitude();

        let scale = [sx, sy, sz];

        i.x = i.x.mul(1.0 / sx);
        i.y = i.y.mul(1.0 / sy);
        i.z = i.z.mul(1.0 / sz);

        let r = cgmath::Quaternion::from(i);
        let rotation = [r.v.x, r.v.y, r.v.z, r.s];

        (translation, rotation, scale)
    }

    pub fn position(&self) -> Point3<f32> {
        Point3::new(self.matrix[12], self.matrix[13], self.matrix[14])
    }

    pub fn rotation(&self) -> Rotation3<f32> {
        let (_, r, _) = self.decomposed();
        Rotation3::from(UnitQuaternion::from_quaternion(Quaternion::new(
            r[3], r[0], r[1], r[2],
        )))
    }
}

impl From<gltf::scene::Transform> for Transform {
    fn from(transform: gltf::scene::Transform) -> Self {
        Self {
            matrix: transform.matrix().into(),
        }
    }
}

impl From<Mat4> for Transform {
    fn from(value: Mat4) -> Self {
        Self {
            matrix: Matrix4::new(
                value.x.x, value.y.x, value.z.x, value.w.x, value.x.y, value.y.y, value.z.y,
                value.w.y, value.x.z, value.y.z, value.z.z, value.w.z, value.x.w, value.y.w,
                value.z.w, value.w.w,
            ),
        }
    }
}

impl From<Transform> for Mat4 {
    fn from(val: Transform) -> Self {
        let x = val.matrix.column(0);
        let y = val.matrix.column(1);
        let z = val.matrix.column(2);
        let w = val.matrix.column(3);

        Mat4::from_cols(
            Vector4::new(x.x, x.y, x.z, x.w),
            Vector4::new(y.x, y.y, y.z, y.w),
            Vector4::new(z.x, z.y, z.z, z.w),
            Vector4::new(w.x, w.y, w.z, w.w),
        )
    }
}

impl Mul for Transform {
    type Output = Transform;

    fn mul(self, rhs: Transform) -> Self::Output {
        Self {
            matrix: self.matrix * rhs.matrix,
        }
    }
}

impl fmt::Debug for Transform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (_, _, scale) = self.decomposed();
        let translation = self.position();
        let (x, y, z) = self.rotation().euler_angles();

        write!(
            f,
            "translation: {:?}\nrotation {:?}\nscale {:?}\n",
            translation,
            [x, y, z].iter().map(|v| v.to_degrees()).collect::<Vec<_>>(),
            scale,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_float_eq() {
        assert!(float_eq(0.0, 0.0));
        assert!(float_eq(0.0001, 0.0001));
        assert!(float_eq(-3.0, -3.0));
        assert!(!float_eq(0.0001, 0.0002));
    }
}
