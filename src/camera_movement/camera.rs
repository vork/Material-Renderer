use cgmath;
use cgmath::prelude::*;
use cgmath::{Basis3, Matrix3, Matrix4, Rad, Vector2, Vector3, Quaternion};

pub fn model_view_projection<T: cgmath::BaseFloat>(
    model: Matrix4<T>,
    view: Matrix4<T>,
    projection: Matrix4<T>
) -> Matrix4<T> {
    projection * view * model
}

pub struct Camera<T: cgmath::BaseFloat> {
    pub position: Vector3<T>,
    pub up: Vector3<T>,
    pub right: Vector3<T>,
    pub forward: Vector3<T>
}

pub struct CameraPerspective<T: cgmath::BaseFloat> {
    pub fov: T,
    pub near_clip: T,
    pub far_clip: T,
    pub aspect_ratio: T
}

impl<T: cgmath::BaseFloat + Copy> Camera<T> {
    pub fn new(position: Vector3<T>) -> Camera<T> {
        Camera {
            position: position,
            right: Vector3::unit_x(),
            up: Vector3::unit_y(),
            forward: Vector3::unit_z()
        }
    }

    pub fn orthogonal(&self) -> Matrix4<T> {
        let p = self.position;
        let r = self.right;
        let u = self.up;
        let f = self.forward;
        let _0 = T::zero();
        Matrix4::from([
            [r[0], u[0], f[0], _0],
            [r[1], u[1], f[1], _0],
            [r[2], u[2], f[2], _0],
            [-r.dot(p), -u.dot(p), -f.dot(p), T::one()],
        ])
    }

    pub fn look_at(&mut self, point: Vector3<T>) {
        self.forward = self.position.normalize() - point.normalize();
        self.update_right();
    }

    pub fn set_yaw_pitch(&mut self, yaw: T, pitch: T) {
        let (y_s, y_c, p_s, p_c) = (yaw.sin(), yaw.cos(), pitch.sin(), pitch.cos());
        self.forward = Vector3::from([y_s * p_c, p_s, y_c * p_c]);
        self.up = Vector3::from([y_s * -p_s, p_c, y_c * -p_s]);
        self.update_right();
    }

    pub fn set_rotation(&mut self, rotation: Quaternion<T>) {
        let forward = Vector3::unit_z();
        let up = Vector3::unit_y();
        self.forward = rotation.rotate_vector(forward);
        self.up = rotation.rotate_vector(up);
        self.update_right();
    }

    pub fn update_right(&mut self) {
        self.right = self.up.cross(self.forward);
    }
}

impl<T: cgmath::BaseFloat + Copy> CameraPerspective<T> {
    pub fn projection(&self) -> Matrix4<T> {
        let _0 = T::zero();
        let _1 = T::one();
        let _2 = _1 + _1;
        let pi: T = Rad::turn_div_2().0;
        let _360 = T::from(360.0f64).unwrap();
        let f = _1 / (self.fov * (pi / _360)).tan();
        let (far, near) = (self.far_clip, self.near_clip);
        Matrix4::from([
            [f / self.aspect_ratio, _0, _0, _0],
            [_0, f, _0, _0],
            [_0, _0, (far + near) / (near - far), -_1],
            [_0, _0, (_2 * far * near) / (near - far), _0]
        ])
    }
}