use cgmath;
use cgmath::prelude::*;
use cgmath::{Basis3, Matrix3, Matrix4, Rad, Vector2, Vector3, Quaternion};

use camera_movement::camera::Camera;

pub struct OrbitZoomCameraSettings<T: cgmath::BaseFloat> {
    pub orbit_speed: T,
    pub pitch_speed: T,
    pub zoom_speed: T,
    pub pan_speed: T
}

impl<T: cgmath::BaseFloat> OrbitZoomCameraSettings<T> {
    pub fn default() -> OrbitZoomCameraSettings<T> {
        OrbitZoomCameraSettings {
            orbit_speed: T::from(0.05f32).unwrap(),
            pitch_speed: T::from(0.1f32).unwrap(),
            pan_speed: T::from(0.1f32).unwrap(),
            zoom_speed: T::from(0.1f32).unwrap(),
        }
    }

    pub fn orbit_speed(self, s: T) -> OrbitZoomCameraSettings<T> {
        OrbitZoomCameraSettings {
            orbit_speed: s,
            .. self
        }
    }

    pub fn pitch_speed(self, s: T) -> OrbitZoomCameraSettings<T> {
        OrbitZoomCameraSettings {
            pitch_speed: s,
            .. self
        }
    }

    pub fn pan_speed(self, s: T) -> OrbitZoomCameraSettings<T> {
        OrbitZoomCameraSettings {
            pan_speed: s,
            .. self
        }
    }

    pub fn zoom_speed(self, s: T) -> OrbitZoomCameraSettings<T> {
        OrbitZoomCameraSettings {
            zoom_speed: s,
            .. self
        }
    }
}

pub struct OrbitCamera<T: cgmath::BaseFloat> {
    prev_mouse: Vector2<T>,
    target: Vector3<T>,
    rotation: Quaternion<T>,
    pitch: T,
    yaw: T,
    distance: T,
    settings: OrbitZoomCameraSettings<T>,
    rotating: bool,
    panning: bool,
    zooming: bool
}

impl<T: cgmath::BaseFloat> OrbitCamera<T> {
    pub fn new(settings: OrbitZoomCameraSettings<T>) -> OrbitCamera<T> {
        OrbitCamera {
            prev_mouse: Vector2::zero(),
            target: Vector3::zero(),
            rotation: Quaternion::one(),
            pitch: T::zero(),
            yaw: T::zero(),
            distance: T::zero(),
            settings: settings,
            rotating: false,
            panning: false,
            zooming: false
        }
    }

    pub fn camera(& self) -> Camera<T> {
        let target_to_cam = self.rotation.rotate_vector(Vector3::new(T::zero(), T::zero(), self.distance));
        let position = self.target + target_to_cam;

        let mut camera = Camera::new(self.target + target_to_cam);
        camera.set_rotation(self.rotation);
        camera
    }

    pub fn get_position(& self) -> Vector3<T> {
        -(self.target + self.rotation.rotate_vector(Vector3::unit_z() * self.distance))
    }

    pub fn set_distance(&mut self, distance: T) -> &mut Self {
        self.distance = distance.max(T::zero());
        self
    }

    pub fn set_rotation(&mut self, rotation: Quaternion<T>) -> &mut Self {
        self.rotation = rotation;
        self
    }

    pub fn set_target(&mut self, target: Vector3<T>) -> &mut Self {
        self.target = target;
        self
    }

    pub fn set_orbit_speed(&mut self, speed: T) -> &mut Self {
        self.settings.orbit_speed = speed;
        self
    }

    pub fn set_zoom_speed(&mut self, speed: T) -> &mut Self {
        self.settings.zoom_speed = speed;
        self
    }

    pub fn set_pan_speed(&mut self, speed: T) -> &mut Self {
        self.settings.pan_speed = speed;
        self
    }

    pub fn rotate_start(&mut self, pos: Vector2<T>) {
        self.rotating = true;
        self.prev_mouse = pos;
    }

    pub fn rotate_end(&mut self) {
        self.rotating = false;
    }

    pub fn pan_start(&mut self, pos: Vector2<T>) {
        self.panning = true;
        self.prev_mouse = pos;
    }

    pub fn pan_end(&mut self) {
        self.panning = false;
    }

    pub fn zoom_start(&mut self, pos: Vector2<T>) {
        self.zooming = true;
        self.prev_mouse = pos;
    }

    pub fn zoom_end(&mut self) {
        self.zooming = false;
    }

    pub fn update(&mut self, cur_mouse: Vector2<T>) {
        if self.rotating {
            let mouse_vec = -(cur_mouse - self.prev_mouse).normalize_to(self.settings.pan_speed);

            self.yaw = self.yaw + mouse_vec.x;
            self.pitch = self.pitch + mouse_vec.y * self.settings.pitch_speed;

            self.rotation = Quaternion::from_axis_angle(Vector3::unit_y(), Rad(self.yaw)) *
                Quaternion::from_axis_angle(Vector3::unit_x(), Rad(self.pitch));
            self.prev_mouse = cur_mouse;
        } else if self.panning {
            // Note that the direction of target point movement is the reverse of the direction of mouse movement
            let mouse_vec = -(cur_mouse - self.prev_mouse).normalize_to(self.settings.pan_speed);

            let left_vec = self.rotation.rotate_vector(Vector3::unit_x() * mouse_vec.x);
            let up_vec = self.rotation.rotate_vector(Vector3::unit_y() * mouse_vec.y);
            self.target += left_vec + up_vec;
            self.prev_mouse = cur_mouse;
        } else if self.zooming {
            let mouse_vec = -(cur_mouse - self.prev_mouse).normalize_to(self.settings.zoom_speed);
            self.distance = (self.distance + mouse_vec.y).max(T::zero());
            self.prev_mouse = cur_mouse;
        }
    }
}

